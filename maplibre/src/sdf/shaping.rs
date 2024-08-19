use crate::sdf::constants::ONE_EM;
use crate::sdf::glyph::{Glyph, GlyphMap, GlyphMetrics, PositionedGlyph, Shaping, WritingModeType};
use crate::sdf::i18n;
use crate::sdf::style_types::{IconTextFitType, SymbolAnchorType, TextJustifyType};
use crate::sdf::tagged_string::{SectionOptions, TaggedString};
use cgmath::num_traits::Pow;
use geo_types::Rect;
use std::collections::HashSet;
use crate::sdf::bidi::BiDi;
use crate::sdf::glyph_atlas::GlyphPositions;

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Padding {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

impl Into<bool> for Padding {
    fn into(self) -> bool {
        self.left != 0. || self.top != 0. || self.right != 0. || self.bottom != 0.
    }
}

// TODO
#[derive(Default)]
struct ImagePosition;
struct ImagePositions;

struct AnchorAlignment {
    horizontalAlign: f64,
    verticalAlign: f64,
}
impl AnchorAlignment {
    fn getAnchorAlignment(anchor: SymbolAnchorType) -> AnchorAlignment {
        let mut result = AnchorAlignment {
            horizontalAlign: 0.5,
            verticalAlign: 0.5,
        };

        match (anchor) {
            SymbolAnchorType::Right
            | SymbolAnchorType::TopRight
            | SymbolAnchorType::BottomRight => {
                result.horizontalAlign = 1.0;
            }

            SymbolAnchorType::Left | SymbolAnchorType::TopLeft | SymbolAnchorType::BottomLeft => {
                result.horizontalAlign = 0.0;
            }
            _ => {}
        }

        match (anchor) {
            SymbolAnchorType::Bottom
            | SymbolAnchorType::BottomLeft
            | SymbolAnchorType::BottomRight => {
                result.verticalAlign = 1.0;
            }

            SymbolAnchorType::Top | SymbolAnchorType::TopLeft | SymbolAnchorType::TopRight => {
                result.verticalAlign = 0.0;
            }

            _ => {}
        }

        return result;
    }
}

// Choose the justification that matches the direction of the TextAnchor
fn getAnchorJustification(anchor: SymbolAnchorType) -> TextJustifyType {
    match (anchor) {
        SymbolAnchorType::Right | SymbolAnchorType::TopRight | SymbolAnchorType::BottomRight => {
            TextJustifyType::Right
        }
        SymbolAnchorType::Left | SymbolAnchorType::TopLeft | SymbolAnchorType::BottomLeft => {
            TextJustifyType::Left
        }
        _ => TextJustifyType::Center,
    }
}

#[derive(Default)]
pub struct PositionedIcon {
    pub image: ImagePosition,
    pub top: f64,
    pub bottom: f64,
    pub left: f64,
    pub right: f64,
    pub collisionPadding: Padding,
}

impl PositionedIcon {
    fn shapeIcon(
        image: ImagePosition,
        iconOffset: [f64; 2],
        iconAnchor: SymbolAnchorType,
    ) -> PositionedIcon {
        let anchorAlign = AnchorAlignment::getAnchorAlignment(iconAnchor);
        let dx = iconOffset[0];
        let dy = iconOffset[1];
        let left = dx - image.displaySize()[0] * anchorAlign.horizontalAlign;
        let right = left + image.displaySize()[0];
        let top = dy - image.displaySize()[1] * anchorAlign.verticalAlign;
        let bottom = top + image.displaySize()[1];

        let mut collisionPadding: Padding = Padding::default();
        if (image.content) {
            let content = *image.content;
            let pixelRatio = image.pixelRatio;
            collisionPadding.left = content.left / pixelRatio;
            collisionPadding.top = content.top / pixelRatio;
            collisionPadding.right = image.displaySize()[0] - content.right / pixelRatio;
            collisionPadding.bottom = image.displaySize()[1] - content.bottom / pixelRatio;
        }

        return PositionedIcon {
            image,
            top,
            bottom,
            left,
            right,
            collisionPadding,
        };
    }

    // Updates shaped icon's bounds based on shaped text's bounds and provided
    // layout properties.
    fn fitIconToText(
        &mut self,
        shapedText: &Shaping,
        textFit: &IconTextFitType,
        padding: &[f64; 4],
        iconOffset: &[f64; 2],
        fontScale: f64,
    ) {
        assert!(textFit != IconTextFitType::None);
        // TODO assert!(shapedText);

        // We don't respect the icon-anchor, because icon-text-fit is set. Instead,
        // the icon will be centered on the text, then stretched in the given
        // dimensions.

        let textLeft = shapedText.left * fontScale;
        let textRight = shapedText.right * fontScale;

        if (textFit == IconTextFitType::Width || textFit == IconTextFitType::Both) {
            // Stretched horizontally to the text width
            self.left = iconOffset[0] + textLeft - padding[3];
            self.right = iconOffset[0] + textRight + padding[1];
        } else {
            // Centered on the text
            self.left = iconOffset[0] + (textLeft + textRight - self.image.displaySize()[0]) / 2.0;
            self.right = self.left + self.image.displaySize()[0];
        }

        let textTop = shapedText.top * fontScale;
        let textBottom = shapedText.bottom * fontScale;
        if (textFit == IconTextFitType::Height || textFit == IconTextFitType::Both) {
            // Stretched vertically to the text height
            self.top = iconOffset[1] + textTop - padding[0];
            self.bottom = iconOffset[1] + textBottom + padding[2];
        } else {
            // Centered on the text
            self.top = iconOffset[1] + (textTop + textBottom - self.image.displaySize()[1]) / 2.0;
            self.bottom = self.top + self.image.displaySize()[1];
        }
    }
}

pub fn getShaping(
    formattedString: &TaggedString,
    maxWidth: f64,
    lineHeight: f64,
    textAnchor: SymbolAnchorType,

    textJustify: TextJustifyType,
    spacing: f64,
    translate: &[f64; 2],
    writingMode: WritingModeType,
    bidi: &BiDi,
    glyphMap: &GlyphMap,
    glyphPositions: GlyphPositions,
    imagePositions: &ImagePositions,
    layoutTextSize: f64,
    layoutTextSizeAtBucketZoomLevel: f64,
    allowVerticalPlacement: bool,
) -> Shaping {
    assert!(layoutTextSize != 0.);
    let mut reorderedLines: Vec<TaggedString> = Vec::new();
    if (formattedString.sectionCount() == 1) {
        let untaggedLines = bidi.processText(
            formattedString.rawText(),
            determineLineBreaks(
                formattedString,
                spacing,
                maxWidth,
                glyphMap,
                imagePositions,
                layoutTextSize,
            ),
        );
        for line in untaggedLines {
            reorderedLines.push(TaggedString::new(line, formattedString.sectionAt(0)));
        }
    } else {
        let processedLines = bidi.processStyledText(
            formattedString.getStyledText(),
            determineLineBreaks(
                formattedString,
                spacing,
                maxWidth,
                glyphMap,
                imagePositions,
                layoutTextSize,
            ),
        );
        for line in processedLines {
            reorderedLines.push(TaggedString::new(line, formattedString.getSections()));
        }
    }
    let shaping = Shaping::new(translate[0], translate[1], writingMode);
    shapeLines(
        shaping,
        &reorderedLines,
        spacing,
        lineHeight,
        textAnchor,
        textJustify,
        writingMode,
        glyphMap,
        &glyphPositions,
        imagePositions,
        layoutTextSizeAtBucketZoomLevel,
        allowVerticalPlacement,
    );

    return shaping;
}

// Zero width space that is used to suggest break points for Japanese labels.
const ZWSP: char = '\u{200b}'; // was char16_t

fn align(
    shaping: &mut Shaping,
    justify: f64,
    horizontalAlign: f64,
    verticalAlign: f64,
    maxLineLength: f64,
    maxLineHeight: f64,
    lineHeight: f64,
    blockHeight: f64,
    lineCount: usize,
) {
    let shiftX = (justify - horizontalAlign) * maxLineLength;
    let mut shiftY = 0.0;

    if (maxLineHeight != lineHeight) {
        shiftY = -blockHeight * verticalAlign - Shaping::yOffset as f64;
    } else {
        shiftY = (-verticalAlign * (lineCount) as f64 + 0.5) * lineHeight;
    }

    for line in &mut shaping.positionedLines {
        for mut positionedGlyph in &mut line.positionedGlyphs {
            positionedGlyph.x += shiftX;
            positionedGlyph.y += shiftY;
        }
    }
}

// justify left = 0, right = 1, center = .5
fn justifyLine(positionedGlyphs: Vec<PositionedGlyph>, justify: f64, lineOffset: f64) {
    if (justify == 0.0 && lineOffset == 0.0) {
        return;
    }

    let lastGlyph = positionedGlyphs.back();
    let lastAdvance: f64 = lastGlyph.metrics.advance * lastGlyph.scale;
    let lineIndent = lastGlyph.x + lastAdvance as f64 * justify;
    for mut positionedGlyph in positionedGlyphs {
        positionedGlyph.x -= lineIndent;
        positionedGlyph.y += lineOffset;
    }
}

fn getGlyphAdvance(
    codePoint: char, // was char16_t
    section: &SectionOptions,
    glyphMap: &GlyphMap,
    imagePositions: &ImagePositions,
    layoutTextSize: f64,
    spacing: f64,
) -> f64 {
    if (!section.imageID) {
        let glyphs = glyphMap.find(section.fontStackHash);
        if (glyphs == glyphMap.end()) {
            return 0.0;
        }
        let it = glyphs.second.find(codePoint);
        if (it == glyphs.second.end() || !it.second) {
            return 0.0;
        }
        return ((*it.second).metrics.advance * section.scale) as f64 + spacing;
    } else {
        let image = imagePositions.find(*section.imageID);
        if (image == imagePositions.end()) {
            return 0.0;
        }
        return image.second.displaySize()[0] * section.scale as f64 * ONE_EM / layoutTextSize
            + spacing;
    }
}

fn determineAverageLineWidth(
    logicalInput: &TaggedString,
    spacing: f64,
    maxWidth: f64,
    glyphMap: &GlyphMap,
    imagePositions: &ImagePositions,
    layoutTextSize: f64,
) -> f64 {
    let mut totalWidth: f64 = 0.;

    for i in 0..logicalInput.length() {
        let section = logicalInput.getSection(i);
        let codePoint: char = logicalInput.getCharCodeAt(i); // was char16_t
        totalWidth += getGlyphAdvance(
            codePoint,
            section,
            glyphMap,
            imagePositions,
            layoutTextSize,
            spacing,
        );
    }

    let targetLineCount = (1.0f64).max((totalWidth / maxWidth).ceil()) as i32;
    return totalWidth / targetLineCount as f64;
}

fn calculateBadness(lineWidth: f64, targetWidth: f64, penalty: f64, isLastBreak: bool) -> f64 {
    let raggedness = (lineWidth - targetWidth).pow(2) as f64;
    if (isLastBreak) {
        // Favor finals lines shorter than average over longer than average
        if (lineWidth < targetWidth) {
            return raggedness / 2.;
        } else {
            return raggedness * 2.;
        }
    }
    if (penalty < 0.) {
        return raggedness - penalty * penalty;
    }
    return raggedness + penalty * penalty;
}

fn calculatePenalty(
    codePoint: char,     /* was char16_t */
    nextCodePoint: char, /* was char16_t */
    penalizableIdeographicBreak: bool,
) -> f64 {
    let mut penalty = 0.;
    // Force break on newline
    if (codePoint == 0x0a as char) {
        penalty -= 10000.;
    }

    // Penalize open parenthesis at end of line
    if (codePoint == 0x28 as char || codePoint == 0xff08 as char) {
        penalty += 50.;
    }

    // Penalize close parenthesis at beginning of line
    if (nextCodePoint == 0x29 as char || nextCodePoint == 0xff09 as char) {
        penalty += 50.;
    }

    // Penalize breaks between characters that allow ideographic breaking because
    // they are less preferable than breaks at spaces (or zero width spaces)
    if (penalizableIdeographicBreak) {
        penalty += 150.;
    }

    return penalty;
}

struct PotentialBreak<'a> {
    pub index: usize,
    pub x: f64,
    pub priorBreak: Option<&'a PotentialBreak<'a>>,
    pub badness: f64,
}

fn evaluateBreak<'a>(
    breakIndex: usize,
    breakX: f64,
    targetWidth: f64,
    potentialBreaks: &'a Vec<PotentialBreak<'a>>,
    penalty: f64,
    isLastBreak: bool,
) -> PotentialBreak<'a> {
    // We could skip evaluating breaks where the line length (breakX - priorBreak.x) > maxWidth
    //  ...but in fact we allow lines longer than maxWidth (if there's no break points)
    //  ...and when targetWidth and maxWidth are close, strictly enforcing maxWidth can give
    //     more lopsided results.

    let mut bestPriorBreak: Option<&PotentialBreak> = None;
    let mut bestBreakBadness: f64 = calculateBadness(breakX, targetWidth, penalty, isLastBreak);
    for potentialBreak in potentialBreaks {
        let lineWidth = breakX - potentialBreak.x;
        let breakBadness =
            calculateBadness(lineWidth, targetWidth, penalty, isLastBreak) + potentialBreak.badness;
        if (breakBadness <= bestBreakBadness) {
            bestPriorBreak = Some(&potentialBreak);
            bestBreakBadness = breakBadness;
        }
    }

    return PotentialBreak {
        index: breakIndex,
        x: breakX,
        priorBreak: bestPriorBreak,
        badness: bestBreakBadness,
    };
}

fn leastBadBreaks(lastLineBreak: &PotentialBreak) -> HashSet<usize> {
    let mut leastBadBreaks: HashSet<usize> = HashSet::from([lastLineBreak.index]);
    let mut priorBreak = lastLineBreak.priorBreak;
    while (priorBreak) {
        leastBadBreaks.insert(priorBreak.index);
        priorBreak = priorBreak.priorBreak;
    }
    return leastBadBreaks;
}

// We determine line breaks based on shaped text in logical order. Working in visual order would be
//  more intuitive, but we can't do that because the visual order may be changed by line breaks!
fn determineLineBreaks(
    logicalInput: &TaggedString,
    spacing: f64,
    maxWidth: f64,
    glyphMap: &GlyphMap,
    imagePositions: &ImagePositions,
    layoutTextSize: f64,
) -> HashSet<usize> {
    if (maxWidth == 0.0) {
        return HashSet::default();
    }

    if (logicalInput.empty()) {
        return HashSet::default();
    }

    let targetWidth = determineAverageLineWidth(
        logicalInput,
        spacing,
        maxWidth,
        glyphMap,
        imagePositions,
        layoutTextSize,
    );

    let mut potentialBreaks: Vec<PotentialBreak> = Vec::new();
    let mut currentX: f64 = 0.;
    // Find first occurance of zero width space (ZWSP) character.
    let hasServerSuggestedBreaks = logicalInput.rawText().find_first_of(ZWSP) != std::string::npos;

    for i in 0..logicalInput.length() {
        let section = logicalInput.getSection(i);
        let codePoint: char = logicalInput.getCharCodeAt(i); // was char16_t
        if (!i18n::isWhitespace(codePoint)) {
            currentX += getGlyphAdvance(
                codePoint,
                section,
                glyphMap,
                imagePositions,
                layoutTextSize,
                spacing,
            );
        }

        // Ideographic characters, spaces, and word-breaking punctuation that
        // often appear without surrounding spaces.
        if (i < logicalInput.length() - 1) {
            let allowsIdeographicBreak = i18n::allowsIdeographicBreaking(codePoint);
            if (section.imageID || allowsIdeographicBreak || i18n::allowsWordBreaking(codePoint)) {
                let penalizableIdeographicBreak =
                    allowsIdeographicBreak && hasServerSuggestedBreaks;
                let nextIndex: usize = i + 1;
                potentialBreaks.push(evaluateBreak(
                    nextIndex,
                    currentX,
                    targetWidth,
                    &potentialBreaks,
                    calculatePenalty(
                        codePoint,
                        logicalInput.getCharCodeAt(nextIndex),
                        penalizableIdeographicBreak,
                    ),
                    false,
                ));
            }
        }
    }

    return leastBadBreaks(evaluateBreak(
        logicalInput.length(),
        currentX,
        targetWidth,
        &potentialBreaks,
        0.,
        true,
    ));
}

fn shapeLines(
    shaping: &mut Shaping,
    lines: &Vec<TaggedString>,
    spacing: f64,
    lineHeight: f64,
    textAnchor: SymbolAnchorType,
    textJustify: TextJustifyType,
    writingMode: WritingModeType,
    glyphMap: &GlyphMap,
    glyphPositions: &GlyphPositions,
    imagePositions: &ImagePositions,
    layoutTextSize: f64,
    allowVerticalPlacement: bool,
) {
    let mut x = 0.0;
    let mut y = Shaping::yOffset as f64;

    let mut maxLineLength = 0.0;
    let mut maxLineHeight = 0.0;

    // TODO was this translated correctly?
    let justify = if textJustify == TextJustifyType::Right {
        1.0
    } else {
        if textJustify == TextJustifyType::Left {
            0.0
        } else {
            0.5
        }
    };

    for line in lines {
        // Collapse whitespace so it doesn't throw off justification
        line.trim();

        let lineMaxScale = line.getMaxScale();
        let maxLineOffset = (lineMaxScale - 1.0) * ONE_EM;
        let mut lineOffset = 0.0;
        shaping.positionedLines.push();
        let positionedLine = shaping.positionedLines.back();
        let positionedGlyphs = positionedLine.positionedGlyphs;

        if (line.empty()) {
            y += lineHeight; // Still need a line feed after empty line
            continue;
        }

        for i in 0..line.length() {
            let sectionIndex = line.getSectionIndex(i);
            let section = line.sectionAt(sectionIndex);
            let codePoint: char = line.getCharCodeAt(i); // was char16_t
            let mut baselineOffset = 0.0;
            let rect: Rect<u16>;
            let mut metrics: GlyphMetrics;
            let mut advance = 0.0;
            let mut verticalAdvance = ONE_EM;
            let sectionScale = section.scale;
            assert!(sectionScale);

            let vertical = !(writingMode == WritingModeType::Horizontal ||
                // Don't verticalize glyphs that have no upright orientation
                // if vertical placement is disabled.
                (!allowVerticalPlacement && !i18n::hasUprightVerticalOrientation(codePoint)) ||
                // If vertical placement is ebabled, don't verticalize glyphs
                // that are from complex text layout script, or whitespaces.
                (allowVerticalPlacement &&
                 (i18n::isWhitespace(codePoint.is_whitespace()) || i18n::isCharInComplexShapingScript(codePoint))));

            if (!section.imageID) {
                let glyphPositionMap = glyphPositions.find(section.fontStackHash);
                if (glyphPositionMap == glyphPositions.end()) {
                    continue;
                }

                let glyphPosition = glyphPositionMap.second.find(codePoint);
                if (glyphPosition != glyphPositionMap.second.end()) {
                    rect = glyphPosition.second.rect;
                    metrics = glyphPosition.second.metrics;
                } else {
                    let glyphs = glyphMap.find(section.fontStackHash);
                    if (glyphs == glyphMap.end()) {
                        continue;
                    }

                    let glyph = glyphs.second.find(codePoint);
                    if (glyph == glyphs.second.end() || !glyph.second) {
                        continue;
                    }
                    metrics = (*glyph.second).metrics;
                }
                advance = metrics.advance as f64;
                // We don't know the baseline, but since we're laying out
                // at 24 points, we can calculate how much it will move when
                // we scale up or down.
                baselineOffset = (lineMaxScale - sectionScale) * ONE_EM;
            } else {
                let image = imagePositions.find(*section.imageID);
                if (image == imagePositions.end()) {
                    continue;
                }
                shaping.iconsInText |= true;
                let displaySize = image.second.displaySize();
                metrics.width = (displaySize[0]) as u32;
                metrics.height = (displaySize[1]) as u32;
                metrics.left = ImagePosition::padding;
                metrics.top = -(Glyph::borderSize as i32);
                metrics.advance = if vertical {
                    metrics.height
                } else {
                    metrics.width
                };
                rect = image.second.paddedRect;

                // If needed, allow to set scale factor for an image using
                // alias "image-scale" that could be alias for "font-scale"
                // when FormattedSection is an image section.
                sectionScale = sectionScale * ONE_EM / layoutTextSize;

                // Difference between one EM and an image size.
                // Aligns bottom of an image to a baseline level.
                let imageOffset = ONE_EM - displaySize[1] * sectionScale as f64;
                baselineOffset = maxLineOffset + imageOffset;
                advance = verticalAdvance = metrics.advance as f64;

                // Difference between height of an image and one EM at max line scale.
                // Pushes current line down if an image size is over 1 EM at max line scale.
                let offset = (if vertical {
                    displaySize[0]
                } else {
                    displaySize[1]
                }) * sectionScale
                    - ONE_EM * lineMaxScale;
                if (offset > 0.0 && offset > lineOffset) {
                    lineOffset = offset;
                }
            }

            if (!vertical) {
                positionedGlyphs.push(
                    codePoint,
                    x,
                    y + baselineOffset as f64,
                    vertical,
                    section.fontStackHash,
                    sectionScale as f64,
                    rect,
                    metrics,
                    section.imageID,
                    sectionIndex,
                );
                x += advance * sectionScale as f64 + spacing;
            } else {
                positionedGlyphs.push(
                    codePoint,
                    x,
                    y + baselineOffset as f64,
                    vertical,
                    section.fontStackHash,
                    sectionScale as f64,
                    rect,
                    metrics,
                    section.imageID,
                    sectionIndex,
                );
                x += verticalAdvance * sectionScale as f64 + spacing;
                shaping.verticalizable |= true;
            }
        }

        // Only justify if we placed at least one glyph
        if (!positionedGlyphs.empty()) {
            let lineLength = x - spacing; // Don't count trailing spacing
            maxLineLength = (lineLength).max(maxLineLength);
            justifyLine(positionedGlyphs, justify, (lineOffset) as f64);
        }

        let currentLineHeight = (lineHeight * lineMaxScale + lineOffset) as f64;
        x = 0.0;
        y += currentLineHeight;
        positionedLine.lineOffset = ((lineOffset).max(maxLineOffset)) as f64;
        maxLineHeight = (currentLineHeight).max(maxLineHeight);
    }

    let anchorAlign = AnchorAlignment::getAnchorAlignment(textAnchor);
    let height = y - Shaping::yOffset as f64;
    align(
        shaping,
        justify,
        anchorAlign.horizontalAlign,
        anchorAlign.verticalAlign,
        maxLineLength,
        maxLineHeight,
        lineHeight,
        height,
        lines.len(),
    );

    // Calculate the bounding box
    shaping.top += -anchorAlign.verticalAlign * height;
    shaping.bottom = shaping.top + height;
    shaping.left += -anchorAlign.horizontalAlign * maxLineLength;
    shaping.right = shaping.left + maxLineLength;
}
