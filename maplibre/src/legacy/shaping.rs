//! Translated from https://github.com/maplibre/maplibre-native/blob/4add9ea/src/mbgl/text/shaping.cpp

use std::collections::BTreeSet;

use cgmath::num_traits::Pow;

use crate::{
    euclid::Rect,
    legacy::{
        bidi::{BiDi, Char16},
        glyph::{
            Glyph, GlyphMap, GlyphMetrics, PositionedGlyph, PositionedLine, Shaping,
            WritingModeType,
        },
        glyph_atlas::GlyphPositions,
        image_atlas::{ImagePosition, ImagePositions},
        style_types::{IconTextFitType, SymbolAnchorType, TextJustifyType},
        tagged_string::{SectionOptions, TaggedString},
        util::{constants::ONE_EM, i18n},
        TileSpace,
    },
};

/// maplibre/maplibre-native#4add9ea original name: Padding
#[derive(Clone, Copy, Default, PartialEq)]
pub struct Padding {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

impl From<Padding> for bool {
    /// maplibre/maplibre-native#4add9ea original name: into
    fn from(val: Padding) -> Self {
        val.left != 0. || val.top != 0. || val.right != 0. || val.bottom != 0.
    }
}

/// maplibre/maplibre-native#4add9ea original name: AnchorAlignment
struct AnchorAlignment {
    horizontal_align: f64,
    vertical_align: f64,
}
impl AnchorAlignment {
    /// maplibre/maplibre-native#4add9ea original name: getAnchorAlignment
    fn get_anchor_alignment(anchor: SymbolAnchorType) -> AnchorAlignment {
        let mut result = AnchorAlignment {
            horizontal_align: 0.5,
            vertical_align: 0.5,
        };

        match anchor {
            SymbolAnchorType::Right
            | SymbolAnchorType::TopRight
            | SymbolAnchorType::BottomRight => {
                result.horizontal_align = 1.0;
            }

            SymbolAnchorType::Left | SymbolAnchorType::TopLeft | SymbolAnchorType::BottomLeft => {
                result.horizontal_align = 0.0;
            }
            _ => {}
        }

        match anchor {
            SymbolAnchorType::Bottom
            | SymbolAnchorType::BottomLeft
            | SymbolAnchorType::BottomRight => {
                result.vertical_align = 1.0;
            }

            SymbolAnchorType::Top | SymbolAnchorType::TopLeft | SymbolAnchorType::TopRight => {
                result.vertical_align = 0.0;
            }

            _ => {}
        }

        result
    }
}

// Choose the justification that matches the direction of the TextAnchor
/// maplibre/maplibre-native#4add9ea original name: getAnchorJustification
pub fn get_anchor_justification(anchor: &SymbolAnchorType) -> TextJustifyType {
    match anchor {
        SymbolAnchorType::Right | SymbolAnchorType::TopRight | SymbolAnchorType::BottomRight => {
            TextJustifyType::Right
        }
        SymbolAnchorType::Left | SymbolAnchorType::TopLeft | SymbolAnchorType::BottomLeft => {
            TextJustifyType::Left
        }
        _ => TextJustifyType::Center,
    }
}

/// maplibre/maplibre-native#4add9ea original name: PositionedIcon
#[derive(Clone)]
pub struct PositionedIcon {
    pub image: ImagePosition,
    pub top: f64,
    pub bottom: f64,
    pub left: f64,
    pub right: f64,
    pub collision_padding: Padding,
}

impl PositionedIcon {
    /// maplibre/maplibre-native#4add9ea original name: shapeIcon
    pub fn shape_icon(
        image: ImagePosition,
        icon_offset: &[f64; 2],
        icon_anchor: SymbolAnchorType,
    ) -> PositionedIcon {
        let anchor_align = AnchorAlignment::get_anchor_alignment(icon_anchor);
        let dx = icon_offset[0];
        let dy = icon_offset[1];
        let left = dx - image.display_size()[0] * anchor_align.horizontal_align;
        let right = left + image.display_size()[0];
        let top = dy - image.display_size()[1] * anchor_align.vertical_align;
        let bottom = top + image.display_size()[1];

        let mut collision_padding: Padding = Padding::default();
        if let Some(content) = &image.content {
            let content = content;
            let pixel_ratio = image.pixel_ratio;
            collision_padding.left = content.left / pixel_ratio;
            collision_padding.top = content.top / pixel_ratio;
            collision_padding.right = image.display_size()[0] - content.right / pixel_ratio;
            collision_padding.bottom = image.display_size()[1] - content.bottom / pixel_ratio;
        }

        PositionedIcon {
            image,
            top,
            bottom,
            left,
            right,
            collision_padding: collision_padding,
        }
    }

    // Updates shaped icon's bounds based on shaped text's bounds and provided
    // layout properties.
    /// maplibre/maplibre-native#4add9ea original name: fitIconToText
    pub fn fit_icon_to_text(
        &mut self,
        shaped_text: &Shaping,
        text_fit: IconTextFitType,
        padding: &[f64; 4],
        icon_offset: &[f64; 2],
        font_scale: f64,
    ) {
        assert!(text_fit != IconTextFitType::None);
        // TODO assert!(shapedText);

        // We don't respect the icon-anchor, because icon-text-fit is set. Instead,
        // the icon will be centered on the text, then stretched in the given
        // dimensions.

        let text_left = shaped_text.left * font_scale;
        let text_right = shaped_text.right * font_scale;

        if text_fit == IconTextFitType::Width || text_fit == IconTextFitType::Both {
            // Stretched horizontally to the text width
            self.left = icon_offset[0] + text_left - padding[3];
            self.right = icon_offset[0] + text_right + padding[1];
        } else {
            // Centered on the text
            self.left =
                icon_offset[0] + (text_left + text_right - self.image.display_size()[0]) / 2.0;
            self.right = self.left + self.image.display_size()[0];
        }

        let text_top = shaped_text.top * font_scale;
        let text_bottom = shaped_text.bottom * font_scale;
        if text_fit == IconTextFitType::Height || text_fit == IconTextFitType::Both {
            // Stretched vertically to the text height
            self.top = icon_offset[1] + text_top - padding[0];
            self.bottom = icon_offset[1] + text_bottom + padding[2];
        } else {
            // Centered on the text
            self.top =
                icon_offset[1] + (text_top + text_bottom - self.image.display_size()[1]) / 2.0;
            self.bottom = self.top + self.image.display_size()[1];
        }
    }
}

/// maplibre/maplibre-native#4add9ea original name: getShaping
pub fn get_shaping(
    formatted_string: &TaggedString,
    max_width: f64,
    line_height: f64,
    text_anchor: SymbolAnchorType,

    text_justify: TextJustifyType,
    spacing: f64,
    translate: &[f64; 2],
    writing_mode: WritingModeType,
    bidi: &BiDi,
    glyph_map: &GlyphMap,
    glyph_positions: &GlyphPositions,
    image_positions: &ImagePositions,
    layout_text_size: f64,
    layout_text_size_at_bucket_zoom_level: f64,
    allow_vertical_placement: bool,
) -> Shaping {
    assert!(layout_text_size != 0.);
    let mut reordered_lines: Vec<TaggedString> = Vec::new();
    if formatted_string.section_count() == 1 {
        let untagged_lines = bidi.process_text(
            formatted_string.raw_text(),
            determine_line_breaks(
                formatted_string,
                spacing,
                max_width,
                glyph_map,
                image_positions,
                layout_text_size,
            ),
        );
        for line in untagged_lines {
            reordered_lines.push(TaggedString::new_from_raw(
                line,
                formatted_string.section_at(0).clone(),
            ));
        }
    } else {
        let processed_lines = bidi.process_styled_text(
            formatted_string.get_styled_text(),
            determine_line_breaks(
                formatted_string,
                spacing,
                max_width,
                glyph_map,
                image_positions,
                layout_text_size,
            ),
        );
        for line in processed_lines {
            reordered_lines.push(TaggedString::new(
                line,
                formatted_string.get_sections().clone(),
            ));
        }
    }
    let mut shaping = Shaping::new(translate[0], translate[1], writing_mode);
    shape_lines(
        &mut shaping,
        &mut reordered_lines,
        spacing,
        line_height,
        text_anchor,
        text_justify,
        writing_mode,
        glyph_map,
        glyph_positions,
        image_positions,
        layout_text_size_at_bucket_zoom_level,
        allow_vertical_placement,
    );

    shaping
}

// Zero width space that is used to suggest break points for Japanese labels.
const ZWSP: Char16 = '\u{200b}' as Char16;

/// maplibre/maplibre-native#4add9ea original name: align
fn align(
    shaping: &mut Shaping,
    justify: f64,
    horizontal_align: f64,
    vertical_align: f64,
    max_line_length: f64,
    max_line_height: f64,
    line_height: f64,
    block_height: f64,
    line_count: usize,
) {
    let shift_x = (justify - horizontal_align) * max_line_length;
    let shift_y = if max_line_height != line_height {
        -block_height * vertical_align - Shaping::Y_OFFSET as f64
    } else {
        (-vertical_align * (line_count) as f64 + 0.5) * line_height
    };

    for line in &mut shaping.positioned_lines {
        for positionedGlyph in &mut line.positioned_glyphs {
            positionedGlyph.x += shift_x;
            positionedGlyph.y += shift_y;
        }
    }
}

// justify left = 0, right = 1, center = .5
/// maplibre/maplibre-native#4add9ea original name: justifyLine
fn justify_line(positioned_glyphs: &mut Vec<PositionedGlyph>, justify: f64, line_offset: f64) {
    if justify == 0.0 && line_offset == 0.0 {
        return;
    }

    let last_glyph = positioned_glyphs.last().unwrap();
    let last_advance: f64 = last_glyph.metrics.advance as f64 * last_glyph.scale;
    let line_indent = last_glyph.x + last_advance * justify;
    for positionedGlyph in positioned_glyphs {
        positionedGlyph.x -= line_indent;
        positionedGlyph.y += line_offset;
    }
}

/// maplibre/maplibre-native#4add9ea original name: getGlyphAdvance
fn get_glyph_advance(
    code_point: Char16,
    section: &SectionOptions,
    glyph_map: &GlyphMap,
    image_positions: &ImagePositions,
    layout_text_size: f64,
    spacing: f64,
) -> f64 {
    if let Some(imageID) = &section.image_id {
        let image = image_positions.get(imageID);
        if image.is_none() {
            return 0.0;
        }
        let image = image.unwrap();
        image.display_size()[0] * section.scale * ONE_EM / layout_text_size + spacing
    } else {
        let glyphs = glyph_map.get(&section.font_stack_hash);
        if glyphs.is_none() {
            return 0.0;
        }
        let glyphs = glyphs.unwrap();
        let it = glyphs.get(&code_point);

        if it.is_none() {
            return 0.0;
        }

        if it.expect("cant be none").is_none() {
            return 0.0;
        }

        return (it
            .expect("cant be none")
            .as_ref()
            .expect("cant be none")
            .metrics
            .advance as f64
            * section.scale)
            + spacing;
    }
}

/// maplibre/maplibre-native#4add9ea original name: determine_average_line_width
fn determine_average_line_width(
    logical_input: &TaggedString,
    spacing: f64,
    max_width: f64,
    glyph_map: &GlyphMap,
    image_positions: &ImagePositions,
    layout_text_size: f64,
) -> f64 {
    let mut total_width: f64 = 0.;

    for i in 0..logical_input.length() {
        let section = logical_input.get_section(i);
        let code_point: Char16 = logical_input.get_char_code_at(i);
        total_width += get_glyph_advance(
            code_point,
            section,
            glyph_map,
            image_positions,
            layout_text_size,
            spacing,
        );
    }

    let target_line_count = (1.0f64).max((total_width / max_width).ceil()) as i32;
    total_width / target_line_count as f64
}

/// maplibre/maplibre-native#4add9ea original name: calculateBadness
fn calculate_badness(line_width: f64, target_width: f64, penalty: f64, is_last_break: bool) -> f64 {
    let raggedness = (line_width - target_width).pow(2);
    if is_last_break {
        // Favor finals lines shorter than average over longer than average
        if line_width < target_width {
            return raggedness / 2.;
        } else {
            return raggedness * 2.;
        }
    }
    if penalty < 0. {
        return raggedness - penalty * penalty;
    }
    raggedness + penalty * penalty
}

/// maplibre/maplibre-native#4add9ea original name: calculatePenalty
fn calculate_penalty(
    code_point: Char16,
    next_code_point: Char16,
    penalizable_ideographic_break: bool,
) -> f64 {
    let mut penalty = 0.;
    // Force break on newline
    if code_point == 0x0au16 {
        penalty -= 10000.;
    }

    // Penalize open parenthesis at end of line
    if code_point == 0x28u16 || code_point == 0xff08u16 {
        penalty += 50.;
    }

    // Penalize close parenthesis at beginning of line
    if next_code_point == 0x29u16 || next_code_point == 0xff09u16 {
        penalty += 50.;
    }

    // Penalize breaks between characters that allow ideographic breaking because
    // they are less preferable than breaks at spaces (or zero width spaces)
    if penalizable_ideographic_break {
        penalty += 150.;
    }

    penalty
}

/// maplibre/maplibre-native#4add9ea original name: PotentialBreak
#[derive(Clone)]
struct PotentialBreak {
    pub index: usize,
    pub x: f64,
    pub prior_break: Option<Box<PotentialBreak>>, // TODO avoid Box
    pub badness: f64,
}

/// maplibre/maplibre-native#4add9ea original name: evaluateBreak
fn evaluate_break(
    break_index: usize,
    break_x: f64,
    target_width: f64,
    potential_breaks: &Vec<PotentialBreak>,
    penalty: f64,
    is_last_break: bool,
) -> PotentialBreak {
    // We could skip evaluating breaks where the line length (breakX - priorBreak.x) > maxWidth
    //  ...but in fact we allow lines longer than maxWidth (if there's no break points)
    //  ...and when targetWidth and maxWidth are close, strictly enforcing maxWidth can give
    //     more lopsided results.

    let mut best_prior_break: Option<Box<PotentialBreak>> = None;
    let mut best_break_badness: f64 =
        calculate_badness(break_x, target_width, penalty, is_last_break);
    for potentialBreak in potential_breaks {
        let line_width = break_x - potentialBreak.x;
        let break_badness = calculate_badness(line_width, target_width, penalty, is_last_break)
            + potentialBreak.badness;
        if break_badness <= best_break_badness {
            best_prior_break = Some(Box::new(potentialBreak.clone()));
            best_break_badness = break_badness;
        }
    }

    PotentialBreak {
        index: break_index,
        x: break_x,
        prior_break: best_prior_break,
        badness: best_break_badness,
    }
}

/// maplibre/maplibre-native#4add9ea original name: leastBadBreaks
fn least_bad_breaks(last_line_break: &PotentialBreak) -> BTreeSet<usize> {
    let mut least_bad_breaks: BTreeSet<usize> = BTreeSet::from([last_line_break.index]);
    let mut prior_break = &last_line_break.prior_break;

    while let Some(priorBreak_) = prior_break {
        least_bad_breaks.insert(priorBreak_.index);
        prior_break = &priorBreak_.prior_break;
    }
    least_bad_breaks
}

// We determine line breaks based on shaped text in logical order. Working in visual order would be
//  more intuitive, but we can't do that because the visual order may be changed by line breaks!
/// maplibre/maplibre-native#4add9ea original name: determine_line_breaks
fn determine_line_breaks(
    logical_input: &TaggedString,
    spacing: f64,
    max_width: f64,
    glyph_map: &GlyphMap,
    image_positions: &ImagePositions,
    layout_text_size: f64,
) -> BTreeSet<usize> {
    if max_width == 0.0 {
        return BTreeSet::default();
    }

    if logical_input.empty() {
        return BTreeSet::default();
    }

    let target_width = determine_average_line_width(
        logical_input,
        spacing,
        max_width,
        glyph_map,
        image_positions,
        layout_text_size,
    );

    let mut potential_breaks: Vec<PotentialBreak> = Vec::new();
    let mut current_x: f64 = 0.;
    // Find first occurance of zero width space (ZWSP) character.
    let has_server_suggested_breaks = logical_input
        .raw_text()
        .as_slice()
        .iter()
        .any(|c| *c == ZWSP);

    for i in 0..logical_input.length() {
        let section = logical_input.get_section(i);
        let code_point: Char16 = logical_input.get_char_code_at(i);
        if !i18n::is_whitespace(code_point) {
            current_x += get_glyph_advance(
                code_point,
                section,
                glyph_map,
                image_positions,
                layout_text_size,
                spacing,
            );
        }

        // Ideographic characters, spaces, and word-breaking punctuation that
        // often appear without surrounding spaces.
        if i < logical_input.length() - 1 {
            let allows_ideographic_break = i18n::allows_ideographic_breaking(code_point);
            if section.image_id.is_some()
                || allows_ideographic_break
                || i18n::allows_word_breaking(code_point)
            {
                let penalizable_ideographic_break =
                    allows_ideographic_break && has_server_suggested_breaks;
                let next_index: usize = i + 1;
                let potential_break = evaluate_break(
                    next_index,
                    current_x,
                    target_width,
                    &potential_breaks,
                    calculate_penalty(
                        code_point,
                        logical_input.get_char_code_at(next_index),
                        penalizable_ideographic_break,
                    ),
                    false,
                );
                potential_breaks.push(potential_break);
            }
        }
    }

    least_bad_breaks(&evaluate_break(
        logical_input.length(),
        current_x,
        target_width,
        &potential_breaks,
        0.,
        true,
    ))
}

/// maplibre/maplibre-native#4add9ea original name: shapeLines
fn shape_lines(
    shaping: &mut Shaping,
    lines: &mut Vec<TaggedString>,
    spacing: f64,
    line_height: f64,
    text_anchor: SymbolAnchorType,
    text_justify: TextJustifyType,
    writing_mode: WritingModeType,
    glyph_map: &GlyphMap,
    glyph_positions: &GlyphPositions,
    image_positions: &ImagePositions,
    layout_text_size: f64,
    allow_vertical_placement: bool,
) {
    let mut x = 0.0;
    let mut y = Shaping::Y_OFFSET as f64;

    let mut max_line_length = 0.0;
    let mut max_line_height = 0.0;

    // TODO was this translated correctly?
    let justify = if text_justify == TextJustifyType::Right {
        1.0
    } else if text_justify == TextJustifyType::Left {
        0.0
    } else {
        0.5
    };

    let n_lines = lines.len();

    for line in lines {
        // Collapse whitespace so it doesn't throw off justification
        line.trim();

        let line_max_scale = line.get_max_scale();
        let max_line_offset = (line_max_scale - 1.0) * ONE_EM;
        let mut line_offset = 0.0;
        shaping.positioned_lines.push(PositionedLine::default());
        let positioned_line = shaping.positioned_lines.last_mut().unwrap();
        let positioned_glyphs = &mut positioned_line.positioned_glyphs;

        if line.empty() {
            y += line_height; // Still need a line feed after empty line
            continue;
        }

        for i in 0..line.length() {
            let section_index = line.get_section_index(i) as usize;
            let section = line.section_at(section_index);
            let code_point: Char16 = line.get_char_code_at(i);
            let mut baseline_offset = 0.0;
            let mut rect: Rect<u16, TileSpace> = Rect::default(); // TODO are these default values fine?
            let mut metrics: GlyphMetrics = GlyphMetrics::default(); // TODO are these default values fine?
            let mut advance = 0.0;
            let mut vertical_advance = ONE_EM;
            let mut section_scale = section.scale;
            assert_ne!(section_scale, 0.0);

            let vertical = !(writing_mode == WritingModeType::Horizontal ||
                // Don't verticalize glyphs that have no upright orientation
                // if vertical placement is disabled.
                (!allow_vertical_placement && !i18n::has_upright_vertical_orientation(code_point)) ||
                // If vertical placement is ebabled, don't verticalize glyphs
                // that are from complex text layout script, or whitespaces.
                (allow_vertical_placement &&
                 (i18n::is_whitespace(code_point) || i18n::is_char_in_complex_shaping_script(code_point))));

            if let Some(imageID) = &section.image_id {
                let image = image_positions.get(imageID);
                if image.is_none() {
                    continue;
                }
                let image = image.expect("is some");

                shaping.icons_in_text |= true;
                let display_size = image.display_size();
                metrics.width = (display_size[0]) as u32;
                metrics.height = (display_size[1]) as u32;
                metrics.left = ImagePosition::PADDING as i32;
                metrics.top = -(Glyph::BORDER_SIZE as i32);
                metrics.advance = if vertical {
                    metrics.height
                } else {
                    metrics.width
                };
                rect = image.padded_rect;

                // If needed, allow to set scale factor for an image using
                // alias "image-scale" that could be alias for "font-scale"
                // when FormattedSection is an image section.
                section_scale = section_scale * ONE_EM / layout_text_size;

                // Difference between one EM and an image size.
                // Aligns bottom of an image to a baseline level.
                let image_offset = ONE_EM - display_size[1] * section_scale;
                baseline_offset = max_line_offset + image_offset;

                vertical_advance = metrics.advance as f64;
                advance = vertical_advance;

                // Difference between height of an image and one EM at max line scale.
                // Pushes current line down if an image size is over 1 EM at max line scale.
                let offset = (if vertical {
                    display_size[0]
                } else {
                    display_size[1]
                }) * section_scale
                    - ONE_EM * line_max_scale;
                if offset > 0.0 && offset > line_offset {
                    line_offset = offset;
                }
            } else {
                let glyph_position_map = glyph_positions.get(&section.font_stack_hash); // TODO was .find
                if glyph_position_map.is_none() {
                    continue;
                }

                let glyph_position_map = glyph_position_map.expect("cant be none");

                let glyph_position = glyph_position_map.get(&code_point);
                if let Some(glyphPosition) = glyph_position {
                    rect = glyphPosition.rect;
                    metrics = glyphPosition.metrics;
                } else {
                    // TODO why would a glyph position not be available but a glyph? Maybe if a glyph bitmap is empty?
                    unreachable!();
                    let glyphs = glyph_map.get(&section.font_stack_hash);
                    if glyphs.is_none() {
                        continue;
                    }
                    let glyphs = glyphs.expect("cant be none");

                    let glyph = glyphs.get(&code_point);

                    if glyph.is_none() {
                        continue;
                    }

                    if glyph.expect("cant be none").is_none() {
                        continue;
                    }

                    metrics =
                        (glyph.expect("cant be none").as_ref().expect("cant be none")).metrics;
                }
                advance = metrics.advance as f64;
                // We don't know the baseline, but since we're laying out
                // at 24 points, we can calculate how much it will move when
                // we scale up or down.
                baseline_offset = (line_max_scale - section_scale) * ONE_EM;
            }

            if !vertical {
                positioned_glyphs.push(PositionedGlyph {
                    glyph: code_point,
                    x,
                    y: y + baseline_offset,
                    vertical,
                    font: section.font_stack_hash,
                    scale: section_scale,
                    rect,
                    metrics,
                    image_id: section.image_id.clone(),
                    section_index,
                });
                x += advance * section_scale + spacing;
            } else {
                positioned_glyphs.push(PositionedGlyph {
                    glyph: code_point,
                    x,
                    y: y + baseline_offset,
                    vertical,
                    font: section.font_stack_hash,
                    scale: section_scale,
                    rect,
                    metrics,
                    image_id: section.image_id.clone(),
                    section_index,
                });
                x += vertical_advance * section_scale + spacing;
                shaping.verticalizable |= true;
            }
        }

        // Only justify if we placed at least one glyph
        if !positioned_glyphs.is_empty() {
            let line_length = x - spacing; // Don't count trailing spacing
            max_line_length = (line_length).max(max_line_length);
            justify_line(positioned_glyphs, justify, line_offset);
        }

        let current_line_height = line_height * line_max_scale + line_offset;
        x = 0.0;
        y += current_line_height;
        positioned_line.line_offset = (line_offset).max(max_line_offset);
        max_line_height = (current_line_height).max(max_line_height);
    }

    let anchor_align = AnchorAlignment::get_anchor_alignment(text_anchor);
    let height = y - Shaping::Y_OFFSET as f64;
    align(
        shaping,
        justify,
        anchor_align.horizontal_align,
        anchor_align.vertical_align,
        max_line_length,
        max_line_height,
        line_height,
        height,
        n_lines,
    );

    // Calculate the bounding box
    shaping.top += -anchor_align.vertical_align * height;
    shaping.bottom = shaping.top + height;
    shaping.left += -anchor_align.horizontal_align * max_line_length;
    shaping.right = shaping.left + max_line_length;
}

#[cfg(test)]
mod test {
    use crate::legacy::{
        bidi::{BiDi, Char16},
        font_stack::FontStackHasher,
        glyph::{Glyph, GlyphMap, Glyphs, WritingModeType},
        glyph_atlas::{GlyphPosition, GlyphPositionMap, GlyphPositions},
        image_atlas::ImagePositions,
        shaping::get_shaping,
        style_types::{SymbolAnchorType, TextJustifyType},
        tagged_string::{SectionOptions, TaggedString},
        util::constants::ONE_EM,
    };

    #[test]
    /// maplibre/maplibre-native#4add9ea original name: Shaping_ZWSP
    fn shaping_zwsp() {
        let mut glyph_position = GlyphPosition::default();
        glyph_position.metrics.width = 18;
        glyph_position.metrics.height = 18;
        glyph_position.metrics.left = 2;
        glyph_position.metrics.top = -8;
        glyph_position.metrics.advance = 21;

        let mut glyph = Glyph::default();
        glyph.id = '中' as Char16;
        glyph.metrics = glyph_position.metrics;

        let bidi = BiDi;
        let font_stack = vec!["font-stack".to_string()];
        let section_options = SectionOptions::new(1.0, font_stack.clone(), None);
        let layout_text_size = 16.0;
        let layout_text_size_at_bucket_zoom_level = 16.0;

        let glyphs: GlyphMap = GlyphMap::from([(
            FontStackHasher::new(&font_stack),
            Glyphs::from([('中' as Char16, Some(glyph))]),
        )]);

        let glyph_positions: GlyphPositions = GlyphPositions::from([(
            FontStackHasher::new(&font_stack),
            GlyphPositionMap::from([('中' as Char16, glyph_position)]),
        )]);
        let image_positions: ImagePositions = ImagePositions::default();

        let test_get_shaping = |string: &TaggedString, max_width_in_chars| {
            return get_shaping(
                string,
                max_width_in_chars as f64 * ONE_EM,
                ONE_EM, // lineHeight
                SymbolAnchorType::Center,
                TextJustifyType::Center,
                0.,          // spacing
                &[0.0, 0.0], // translate
                WritingModeType::Horizontal,
                &bidi,
                &glyphs,
                &glyph_positions,
                &image_positions,
                layout_text_size,
                layout_text_size_at_bucket_zoom_level,
                /*allowVerticalPlacement*/ false,
            );
        };

        // 3 lines
        // 中中中中中中
        // 中中中中中中
        // 中中
        {
            let string = TaggedString::new_from_raw(
                "中中\u{200b}中中\u{200b}中中\u{200b}中中中中中中\u{200b}中中".into(),
                section_options.clone(),
            );
            let shaping = test_get_shaping(&string, 5);
            assert_eq!(shaping.positioned_lines.len(), 3);
            assert_eq!(shaping.top, -36.);
            assert_eq!(shaping.bottom, 36.);
            assert_eq!(shaping.left, -63.);
            assert_eq!(shaping.right, 63.);
            assert_eq!(shaping.writing_mode, WritingModeType::Horizontal);
        }

        // 2 lines
        // 中中
        // 中
        {
            let string =
                TaggedString::new_from_raw("中中\u{200b}中".into(), section_options.clone());
            let shaping = test_get_shaping(&string, 1);
            assert_eq!(shaping.positioned_lines.len(), 2);
            assert_eq!(shaping.top, -24.);
            assert_eq!(shaping.bottom, 24.);
            assert_eq!(shaping.left, -21.);
            assert_eq!(shaping.right, 21.);
            assert_eq!(shaping.writing_mode, WritingModeType::Horizontal);
        }

        // 1 line
        // 中中
        {
            let string = TaggedString::new_from_raw("中中\u{200b}".into(), section_options.clone());
            let shaping = test_get_shaping(&string, 2);
            assert_eq!(shaping.positioned_lines.len(), 1);
            assert_eq!(shaping.top, -12.);
            assert_eq!(shaping.bottom, 12.);
            assert_eq!(shaping.left, -21.);
            assert_eq!(shaping.right, 21.);
            assert_eq!(shaping.writing_mode, WritingModeType::Horizontal);
        }

        // 5 'new' lines.
        {
            let string = TaggedString::new_from_raw(
                "\u{200b}\u{200b}\u{200b}\u{200b}\u{200b}".into(),
                section_options.clone(),
            );
            let shaping = test_get_shaping(&string, 1);
            assert_eq!(shaping.positioned_lines.len(), 5);
            assert_eq!(shaping.top, -60.);
            assert_eq!(shaping.bottom, 60.);
            assert_eq!(shaping.left, 0.);
            assert_eq!(shaping.right, 0.);
            assert_eq!(shaping.writing_mode, WritingModeType::Horizontal);
        }
    }
}
