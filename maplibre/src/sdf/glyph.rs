// Except for to-do comments this file was fully translated

use std::collections::{BTreeMap, BTreeSet, HashSet};

use bitflags::bitflags;

use crate::{
    euclid::Rect,
    sdf::{
        bidi::Char16,
        font_stack::{FontStack, FontStackHash},
        glyph_range::GlyphRange,
        TileSpace,
    },
};

// TODO structs
#[derive(Default)]
pub struct AlphaImage;

pub type GlyphID = Char16;
pub type GlyphIDs = BTreeSet<GlyphID>;

// Note: this only works for the BMP
pub fn getGlyphRange(glyph: GlyphID) -> GlyphRange {
    let mut start: u16 = (glyph / 256) * 256;
    let mut end = (start + 255);
    if (start > 65280) {
        start = 65280;
    }
    if (end > 65535) {
        end = 65535;
    }
    return start..end;
}

#[derive(PartialEq, Default, Copy, Clone)]
pub struct GlyphMetrics {
    pub width: u32,
    pub height: u32,
    pub left: i32,
    pub top: i32,
    pub advance: u32,
}

#[derive(Default)]
pub struct Glyph {
    // We're using this value throughout the Mapbox GL ecosystem. If this is
    // different, the glyphs also need to be reencoded.
    pub id: GlyphID,

    // A signed distance field of the glyph with a border (see above).
    pub bitmap: AlphaImage,

    // Glyph metrics
    pub metrics: GlyphMetrics,
}

impl Glyph {
    pub const borderSize: u8 = 3;
}

pub type Glyphs = BTreeMap<GlyphID, Option<Glyph>>;
pub type GlyphMap = BTreeMap<FontStackHash, Glyphs>;

#[derive(Clone)]
pub struct PositionedGlyph {
    pub glyph: GlyphID,
    pub x: f64,
    pub y: f64,
    pub vertical: bool,
    pub font: FontStackHash,
    pub scale: f64,
    pub rect: Rect<u16, TileSpace>,
    pub metrics: GlyphMetrics,
    pub imageID: Option<String>,
    // Maps positioned glyph to TaggedString section
    pub sectionIndex: usize,
}

#[derive(Default, Clone)]
pub struct PositionedLine {
    pub positionedGlyphs: Vec<PositionedGlyph>,
    pub lineOffset: f64,
}

#[derive(Clone, Default)]
pub struct Shaping {
    pub positionedLines: Vec<PositionedLine>,
    pub top: f64,
    pub bottom: f64,
    pub left: f64,
    pub right: f64,
    pub writingMode: WritingModeType,

    pub verticalizable: bool,
    pub iconsInText: bool,
}
impl Shaping {
    // The y offset *should* be part of the font metadata.
    pub const yOffset: i32 = -17;

    pub fn new(x: f64, y: f64, writingMode_: WritingModeType) -> Self {
        Self {
            positionedLines: vec![],
            top: y,
            bottom: y,
            left: x,
            right: x,
            writingMode: writingMode_,
            verticalizable: false,
            iconsInText: false,
        }
    }
    pub fn isAnyLineNotEmpty(&self) -> bool {
        self.positionedLines
            .iter()
            .any(|line| !line.positionedGlyphs.is_empty())
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct WritingModeType: u8 {
        const None = 0;
        const Horizontal = 1 << 0;
        const Vertical = 1 << 1;
    }
}

impl Default for WritingModeType {
    fn default() -> Self {
        WritingModeType::None
    }
}

pub type GlyphDependencies = BTreeMap<FontStack, GlyphIDs>;
pub type GlyphRangeDependencies = BTreeMap<FontStack, HashSet<GlyphRange>>;
