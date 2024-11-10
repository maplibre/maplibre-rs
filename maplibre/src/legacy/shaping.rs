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

impl Into<bool> for Padding {
    /// maplibre/maplibre-native#4add9ea original name: into
    fn into(self) -> bool {
        self.left != 0. || self.top != 0. || self.right != 0. || self.bottom != 0.
    }
}

/// maplibre/maplibre-native#4add9ea original name: AnchorAlignment
struct AnchorAlignment {
    horizontalAlign: f64,
    verticalAlign: f64,
}
impl AnchorAlignment {
    /// maplibre/maplibre-native#4add9ea original name: getAnchorAlignment
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
/// maplibre/maplibre-native#4add9ea original name: getAnchorJustification
pub fn getAnchorJustification(anchor: &SymbolAnchorType) -> TextJustifyType {
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

/// maplibre/maplibre-native#4add9ea original name: PositionedIcon
#[derive(Clone)]
pub struct PositionedIcon {
    pub image: ImagePosition,
    pub top: f64,
    pub bottom: f64,
    pub left: f64,
    pub right: f64,
    pub collisionPadding: Padding,
}

impl PositionedIcon {
    /// maplibre/maplibre-native#4add9ea original name: shapeIcon
    pub fn shapeIcon(
        image: ImagePosition,
        iconOffset: &[f64; 2],
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
        if let Some(content) = (&image.content) {
            let content = content;
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
    /// maplibre/maplibre-native#4add9ea original name: fitIconToText
    pub fn fitIconToText(
        &mut self,
        shapedText: &Shaping,
        textFit: IconTextFitType,
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

/// maplibre/maplibre-native#4add9ea original name: getShaping
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
    glyphPositions: &GlyphPositions,
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
            reorderedLines.push(TaggedString::new_from_raw(
                line,
                formattedString.sectionAt(0).clone(),
            ));
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
            reorderedLines.push(TaggedString::new(
                line,
                formattedString.getSections().clone(),
            ));
        }
    }
    let mut shaping = Shaping::new(translate[0], translate[1], writingMode);
    shapeLines(
        &mut shaping,
        &mut reorderedLines,
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
const ZWSP: Char16 = '\u{200b}' as Char16;

/// maplibre/maplibre-native#4add9ea original name: align
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
    let shiftY = if (maxLineHeight != lineHeight) {
        -blockHeight * verticalAlign - Shaping::yOffset as f64
    } else {
        (-verticalAlign * (lineCount) as f64 + 0.5) * lineHeight
    };

    for line in &mut shaping.positionedLines {
        for positionedGlyph in &mut line.positionedGlyphs {
            positionedGlyph.x += shiftX;
            positionedGlyph.y += shiftY;
        }
    }
}

// justify left = 0, right = 1, center = .5
/// maplibre/maplibre-native#4add9ea original name: justifyLine
fn justifyLine(positionedGlyphs: &mut Vec<PositionedGlyph>, justify: f64, lineOffset: f64) {
    if (justify == 0.0 && lineOffset == 0.0) {
        return;
    }

    let lastGlyph = positionedGlyphs.last().unwrap();
    let lastAdvance: f64 = lastGlyph.metrics.advance as f64 * lastGlyph.scale;
    let lineIndent = lastGlyph.x + lastAdvance as f64 * justify;
    for positionedGlyph in positionedGlyphs {
        positionedGlyph.x -= lineIndent;
        positionedGlyph.y += lineOffset;
    }
}

/// maplibre/maplibre-native#4add9ea original name: getGlyphAdvance
fn getGlyphAdvance(
    codePoint: Char16,
    section: &SectionOptions,
    glyphMap: &GlyphMap,
    imagePositions: &ImagePositions,
    layoutTextSize: f64,
    spacing: f64,
) -> f64 {
    if let Some(imageID) = &section.imageID {
        let image = imagePositions.get(imageID);
        if (image.is_none()) {
            return 0.0;
        }
        let image = image.unwrap();
        return image.displaySize()[0] * section.scale as f64 * ONE_EM / layoutTextSize + spacing;
    } else {
        let glyphs = glyphMap.get(&section.fontStackHash);
        if (glyphs.is_none()) {
            return 0.0;
        }
        let glyphs = glyphs.unwrap();
        let it = glyphs.get(&codePoint);

        if (it.is_none()) {
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
            * section.scale) as f64
            + spacing;
    }
}

/// maplibre/maplibre-native#4add9ea original name: determineAverageLineWidth
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
        let codePoint: Char16 = logicalInput.getCharCodeAt(i);
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

/// maplibre/maplibre-native#4add9ea original name: calculateBadness
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

/// maplibre/maplibre-native#4add9ea original name: calculatePenalty
fn calculatePenalty(
    codePoint: Char16,
    nextCodePoint: Char16,
    penalizableIdeographicBreak: bool,
) -> f64 {
    let mut penalty = 0.;
    // Force break on newline
    if (codePoint == 0x0au16) {
        penalty -= 10000.;
    }

    // Penalize open parenthesis at end of line
    if (codePoint == 0x28u16 || codePoint == 0xff08u16) {
        penalty += 50.;
    }

    // Penalize close parenthesis at beginning of line
    if (nextCodePoint == 0x29u16 || nextCodePoint == 0xff09u16) {
        penalty += 50.;
    }

    // Penalize breaks between characters that allow ideographic breaking because
    // they are less preferable than breaks at spaces (or zero width spaces)
    if (penalizableIdeographicBreak) {
        penalty += 150.;
    }

    return penalty;
}

/// maplibre/maplibre-native#4add9ea original name: PotentialBreak
#[derive(Clone)]
struct PotentialBreak {
    pub index: usize,
    pub x: f64,
    pub priorBreak: Option<Box<PotentialBreak>>, // TODO avoid Box
    pub badness: f64,
}

/// maplibre/maplibre-native#4add9ea original name: evaluateBreak
fn evaluateBreak(
    breakIndex: usize,
    breakX: f64,
    targetWidth: f64,
    potentialBreaks: &Vec<PotentialBreak>,
    penalty: f64,
    isLastBreak: bool,
) -> PotentialBreak {
    // We could skip evaluating breaks where the line length (breakX - priorBreak.x) > maxWidth
    //  ...but in fact we allow lines longer than maxWidth (if there's no break points)
    //  ...and when targetWidth and maxWidth are close, strictly enforcing maxWidth can give
    //     more lopsided results.

    let mut bestPriorBreak: Option<Box<PotentialBreak>> = None;
    let mut bestBreakBadness: f64 = calculateBadness(breakX, targetWidth, penalty, isLastBreak);
    for potentialBreak in potentialBreaks {
        let lineWidth = breakX - potentialBreak.x;
        let breakBadness =
            calculateBadness(lineWidth, targetWidth, penalty, isLastBreak) + potentialBreak.badness;
        if (breakBadness <= bestBreakBadness) {
            bestPriorBreak = Some(Box::new(potentialBreak.clone()));
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

/// maplibre/maplibre-native#4add9ea original name: leastBadBreaks
fn leastBadBreaks(lastLineBreak: &PotentialBreak) -> BTreeSet<usize> {
    let mut leastBadBreaks: BTreeSet<usize> = BTreeSet::from([lastLineBreak.index]);
    let mut priorBreak = &lastLineBreak.priorBreak;

    while let Some(priorBreak_) = priorBreak {
        leastBadBreaks.insert(priorBreak_.index);
        priorBreak = &priorBreak_.priorBreak;
    }
    return leastBadBreaks;
}

// We determine line breaks based on shaped text in logical order. Working in visual order would be
//  more intuitive, but we can't do that because the visual order may be changed by line breaks!
/// maplibre/maplibre-native#4add9ea original name: determineLineBreaks
fn determineLineBreaks(
    logicalInput: &TaggedString,
    spacing: f64,
    maxWidth: f64,
    glyphMap: &GlyphMap,
    imagePositions: &ImagePositions,
    layoutTextSize: f64,
) -> BTreeSet<usize> {
    if (maxWidth == 0.0) {
        return BTreeSet::default();
    }

    if (logicalInput.empty()) {
        return BTreeSet::default();
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
    let hasServerSuggestedBreaks = logicalInput.rawText().as_slice().iter().any(|c| *c == ZWSP);

    for i in 0..logicalInput.length() {
        let section = logicalInput.getSection(i);
        let codePoint: Char16 = logicalInput.getCharCodeAt(i);
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
            if (section.imageID.is_some()
                || allowsIdeographicBreak
                || i18n::allowsWordBreaking(codePoint))
            {
                let penalizableIdeographicBreak =
                    allowsIdeographicBreak && hasServerSuggestedBreaks;
                let nextIndex: usize = i + 1;
                let potential_break = evaluateBreak(
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
                );
                potentialBreaks.push(potential_break);
            }
        }
    }

    return leastBadBreaks(&evaluateBreak(
        logicalInput.length(),
        currentX,
        targetWidth,
        &potentialBreaks,
        0.,
        true,
    ));
}

/// maplibre/maplibre-native#4add9ea original name: shapeLines
fn shapeLines(
    shaping: &mut Shaping,
    lines: &mut Vec<TaggedString>,
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

    let n_lines = lines.len();

    for line in lines {
        // Collapse whitespace so it doesn't throw off justification
        line.trim();

        let lineMaxScale = line.getMaxScale();
        let maxLineOffset = (lineMaxScale - 1.0) * ONE_EM;
        let mut lineOffset = 0.0;
        shaping.positionedLines.push(PositionedLine::default());
        let positionedLine = shaping.positionedLines.last_mut().unwrap();
        let positionedGlyphs = &mut positionedLine.positionedGlyphs;

        if (line.empty()) {
            y += lineHeight; // Still need a line feed after empty line
            continue;
        }

        for i in 0..line.length() {
            let sectionIndex = line.getSectionIndex(i) as usize;
            let section = line.sectionAt(sectionIndex);
            let codePoint: Char16 = line.getCharCodeAt(i);
            let mut baselineOffset = 0.0;
            let mut rect: Rect<u16, TileSpace> = Rect::default(); // TODO are these default values fine?
            let mut metrics: GlyphMetrics = GlyphMetrics::default(); // TODO are these default values fine?
            let mut advance = 0.0;
            let mut verticalAdvance = ONE_EM;
            let mut sectionScale = section.scale;
            assert_ne!(sectionScale, 0.0);

            let vertical = !(writingMode == WritingModeType::Horizontal ||
                // Don't verticalize glyphs that have no upright orientation
                // if vertical placement is disabled.
                (!allowVerticalPlacement && !i18n::hasUprightVerticalOrientation(codePoint)) ||
                // If vertical placement is ebabled, don't verticalize glyphs
                // that are from complex text layout script, or whitespaces.
                (allowVerticalPlacement &&
                 (i18n::isWhitespace(codePoint) || i18n::isCharInComplexShapingScript(codePoint))));

            if let Some(imageID) = &section.imageID {
                let image = imagePositions.get(imageID);
                if (image.is_none()) {
                    continue;
                }
                let image = image.expect("is some");

                shaping.iconsInText |= true;
                let displaySize = image.displaySize();
                metrics.width = (displaySize[0]) as u32;
                metrics.height = (displaySize[1]) as u32;
                metrics.left = ImagePosition::padding as i32;
                metrics.top = -(Glyph::borderSize as i32);
                metrics.advance = if vertical {
                    metrics.height
                } else {
                    metrics.width
                };
                rect = image.paddedRect;

                // If needed, allow to set scale factor for an image using
                // alias "image-scale" that could be alias for "font-scale"
                // when FormattedSection is an image section.
                sectionScale = sectionScale * ONE_EM / layoutTextSize;

                // Difference between one EM and an image size.
                // Aligns bottom of an image to a baseline level.
                let imageOffset = ONE_EM - displaySize[1] * sectionScale as f64;
                baselineOffset = maxLineOffset + imageOffset;

                verticalAdvance = metrics.advance as f64;
                advance = verticalAdvance;

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
            } else {
                let glyphPositionMap = glyphPositions.get(&section.fontStackHash); // TODO was .find
                if (glyphPositionMap.is_none()) {
                    continue;
                }

                let glyphPositionMap = glyphPositionMap.expect("cant be none");

                let glyphPosition = glyphPositionMap.get(&codePoint);
                if let Some(glyphPosition) = glyphPosition {
                    rect = glyphPosition.rect;
                    metrics = glyphPosition.metrics.clone();
                } else {
                    // TODO why would a glyph position not be available but a glyph? Maybe if a glyph bitmap is empty?
                    unreachable!();
                    let glyphs = glyphMap.get(&section.fontStackHash);
                    if (glyphs.is_none()) {
                        continue;
                    }
                    let glyphs = glyphs.expect("cant be none");

                    let glyph = glyphs.get(&codePoint);

                    if (glyph.is_none()) {
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
                baselineOffset = (lineMaxScale - sectionScale) * ONE_EM;
            }

            if (!vertical) {
                positionedGlyphs.push(PositionedGlyph {
                    glyph: codePoint,
                    x,
                    y: y + baselineOffset as f64,
                    vertical,
                    font: section.fontStackHash,
                    scale: sectionScale as f64,
                    rect,
                    metrics,
                    imageID: section.imageID.clone(),
                    sectionIndex,
                });
                x += advance * sectionScale as f64 + spacing;
            } else {
                positionedGlyphs.push(PositionedGlyph {
                    glyph: codePoint,
                    x,
                    y: y + baselineOffset as f64,
                    vertical,
                    font: section.fontStackHash,
                    scale: sectionScale as f64,
                    rect,
                    metrics,
                    imageID: section.imageID.clone(),
                    sectionIndex,
                });
                x += verticalAdvance * sectionScale as f64 + spacing;
                shaping.verticalizable |= true;
            }
        }

        // Only justify if we placed at least one glyph
        if (!positionedGlyphs.is_empty()) {
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
        n_lines,
    );

    // Calculate the bounding box
    shaping.top += -anchorAlign.verticalAlign * height;
    shaping.bottom = shaping.top + height;
    shaping.left += -anchorAlign.horizontalAlign * maxLineLength;
    shaping.right = shaping.left + maxLineLength;
}

#[cfg(test)]
mod test {
    use crate::legacy::{
        bidi::{BiDi, Char16},
        font_stack::FontStackHasher,
        glyph::{Glyph, GlyphMap, Glyphs, WritingModeType},
        glyph_atlas::{GlyphPosition, GlyphPositionMap, GlyphPositions},
        image_atlas::ImagePositions,
        shaping::getShaping,
        style_types::{SymbolAnchorType, TextJustifyType},
        tagged_string::{SectionOptions, TaggedString},
        util::constants::ONE_EM,
    };

    #[test]
    /// maplibre/maplibre-native#4add9ea original name: Shaping_ZWSP
    fn Shaping_ZWSP() {
        let mut glyphPosition = GlyphPosition::default();
        glyphPosition.metrics.width = 18;
        glyphPosition.metrics.height = 18;
        glyphPosition.metrics.left = 2;
        glyphPosition.metrics.top = -8;
        glyphPosition.metrics.advance = 21;

        let mut glyph = Glyph::default();
        glyph.id = '中' as Char16;
        glyph.metrics = glyphPosition.metrics;

        let bidi = BiDi;
        let fontStack = vec!["font-stack".to_string()];
        let sectionOptions = SectionOptions::new(1.0, fontStack.clone(), None);
        let layoutTextSize = 16.0;
        let layoutTextSizeAtBucketZoomLevel = 16.0;

        let glyphs: GlyphMap = GlyphMap::from([(
            FontStackHasher::new(&fontStack),
            Glyphs::from([('中' as Char16, Some(glyph))]),
        )]);

        let glyphPositions: GlyphPositions = GlyphPositions::from([(
            FontStackHasher::new(&fontStack),
            GlyphPositionMap::from([('中' as Char16, glyphPosition)]),
        )]);
        let imagePositions: ImagePositions = ImagePositions::default();

        let testGetShaping = |string: &TaggedString, maxWidthInChars| {
            return getShaping(
                string,
                maxWidthInChars as f64 * ONE_EM,
                ONE_EM, // lineHeight
                SymbolAnchorType::Center,
                TextJustifyType::Center,
                0.,          // spacing
                &[0.0, 0.0], // translate
                WritingModeType::Horizontal,
                &bidi,
                &glyphs,
                &glyphPositions,
                &imagePositions,
                layoutTextSize,
                layoutTextSizeAtBucketZoomLevel,
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
                sectionOptions.clone(),
            );
            let shaping = testGetShaping(&string, 5);
            assert_eq!(shaping.positionedLines.len(), 3);
            assert_eq!(shaping.top, -36.);
            assert_eq!(shaping.bottom, 36.);
            assert_eq!(shaping.left, -63.);
            assert_eq!(shaping.right, 63.);
            assert_eq!(shaping.writingMode, WritingModeType::Horizontal);
        }

        // 2 lines
        // 中中
        // 中
        {
            let string =
                TaggedString::new_from_raw("中中\u{200b}中".into(), sectionOptions.clone());
            let shaping = testGetShaping(&string, 1);
            assert_eq!(shaping.positionedLines.len(), 2);
            assert_eq!(shaping.top, -24.);
            assert_eq!(shaping.bottom, 24.);
            assert_eq!(shaping.left, -21.);
            assert_eq!(shaping.right, 21.);
            assert_eq!(shaping.writingMode, WritingModeType::Horizontal);
        }

        // 1 line
        // 中中
        {
            let string = TaggedString::new_from_raw("中中\u{200b}".into(), sectionOptions.clone());
            let shaping = testGetShaping(&string, 2);
            assert_eq!(shaping.positionedLines.len(), 1);
            assert_eq!(shaping.top, -12.);
            assert_eq!(shaping.bottom, 12.);
            assert_eq!(shaping.left, -21.);
            assert_eq!(shaping.right, 21.);
            assert_eq!(shaping.writingMode, WritingModeType::Horizontal);
        }

        // 5 'new' lines.
        {
            let string = TaggedString::new_from_raw(
                "\u{200b}\u{200b}\u{200b}\u{200b}\u{200b}".into(),
                sectionOptions.clone(),
            );
            let shaping = testGetShaping(&string, 1);
            assert_eq!(shaping.positionedLines.len(), 5);
            assert_eq!(shaping.top, -60.);
            assert_eq!(shaping.bottom, 60.);
            assert_eq!(shaping.left, 0.);
            assert_eq!(shaping.right, 0.);
            assert_eq!(shaping.writingMode, WritingModeType::Horizontal);
        }
    }
}
