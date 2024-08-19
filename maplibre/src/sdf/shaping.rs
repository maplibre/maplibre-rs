use crate::sdf::glyph::Shaping;

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
struct SymbolAnchorType;
struct TextJustifyType;
struct ImagePosition;
struct IconTextFitType;
struct TaggedString;
struct WritingModeType;
struct BiDi;
struct GlyphMap;
struct GlyphPositions;
struct ImagePositions;

struct AnchorAlignment {
    horizontalAlign: f64,
    verticalAlign: f64,
}
impl AnchorAlignment {
    fn getAnchorAlignment(anchor: SymbolAnchorType) -> AnchorAlignment {
        todo!()
    }
}

// Choose the justification that matches the direction of the TextAnchor
fn getAnchorJustification(anchor: SymbolAnchorType) -> TextJustifyType {
    todo!()
}

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
        image_position: &ImagePosition,
        iconOffset: [f64; 2],
        iconAnchor: SymbolAnchorType,
    ) -> PositionedIcon {
        todo!();
    }

    // Updates shaped icon's bounds based on shaped text's bounds and provided
    // layout properties.
    fn fitIconToText(
        &self,
        shapedText: &Shaping,
        textFit: &IconTextFitType,
        padding: &[f64; 4],
        iconOffset: &[f64; 2],
        fontScale: f64,
    ) {
    }
}

pub fn getShaping(
    string: &TaggedString,
    maxWidth: f64,
    lineHeight: f64,
    textAnchor: SymbolAnchorType,

    textJustify: TextJustifyType,
    spacing: f64,
    translate: &[f64; 2],
    writing_mode: WritingModeType,
    bidi: &BiDi,
    glyphMap: &GlyphMap,
    glyphPositions: GlyphPositions,
    imagePositions: &ImagePositions,
    layoutTextSize: f64,
    layoutTextSizeAtBucketZoomLevel: f64,
    allowVerticalPlacement: bool,
) -> Shaping {
    todo!()
}
