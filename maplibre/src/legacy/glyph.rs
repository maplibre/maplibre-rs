//! Translated from https://github.com/maplibre/maplibre-native/blob/4add9ea/src/mbgl/text/glyph.cpp

use std::collections::{BTreeMap, BTreeSet, HashSet};

use bitflags::bitflags;

use crate::{
    euclid::Rect,
    legacy::{
        bidi::Char16,
        font_stack::{FontStack, FontStackHash},
        glyph_range::GlyphRange,
        TileSpace,
    },
};

// TODO structs
/// maplibre/maplibre-native#4add9ea original name: AlphaImage
#[derive(Default)]
pub struct AlphaImage;

/// maplibre/maplibre-native#4add9ea original name: GlyphID
pub type GlyphID = Char16;
/// maplibre/maplibre-native#4add9ea original name: GlyphIDs
pub type GlyphIDs = BTreeSet<GlyphID>;

// Note: this only works for the BMP
/// maplibre/maplibre-native#4add9ea original name: getGlyphRange
pub fn getGlyphRange(glyph: GlyphID) -> GlyphRange {
    let mut start: u16 = (glyph / 256) * 256;
    let mut end = start + 255;
    if start > 65280 {
        start = 65280;
    }
    if end > 65535 {
        end = 65535;
    }
    start..end
}

/// maplibre/maplibre-native#4add9ea original name: GlyphMetrics
#[derive(PartialEq, Default, Copy, Clone)]
pub struct GlyphMetrics {
    pub width: u32,
    pub height: u32,
    pub left: i32,
    pub top: i32,
    pub advance: u32,
}

/// maplibre/maplibre-native#4add9ea original name: Glyph
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

/// maplibre/maplibre-native#4add9ea original name: Glyphs
pub type Glyphs = BTreeMap<GlyphID, Option<Glyph>>;
/// maplibre/maplibre-native#4add9ea original name: GlyphMap
pub type GlyphMap = BTreeMap<FontStackHash, Glyphs>;

/// maplibre/maplibre-native#4add9ea original name: PositionedGlyph
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

/// maplibre/maplibre-native#4add9ea original name: PositionedLine
#[derive(Default, Clone)]
pub struct PositionedLine {
    pub positionedGlyphs: Vec<PositionedGlyph>,
    pub lineOffset: f64,
}

/// maplibre/maplibre-native#4add9ea original name: Shaping
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

    /// maplibre/maplibre-native#4add9ea original name: new
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
    /// maplibre/maplibre-native#4add9ea original name: isAnyLineNotEmpty
    pub fn isAnyLineNotEmpty(&self) -> bool {
        self.positionedLines
            .iter()
            .any(|line| !line.positionedGlyphs.is_empty())
    }
}

bitflags! {
    /// maplibre/maplibre-native#4add9ea original name: WritingModeType:
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct WritingModeType: u8 {
        const None = 0;
        const Horizontal = 1 << 0;
        const Vertical = 1 << 1;
    }
}

impl Default for WritingModeType {
    /// maplibre/maplibre-native#4add9ea original name: default
    fn default() -> Self {
        WritingModeType::None
    }
}

/// maplibre/maplibre-native#4add9ea original name: GlyphDependencies
pub type GlyphDependencies = BTreeMap<FontStack, GlyphIDs>;
/// maplibre/maplibre-native#4add9ea original name: GlyphRangeDependencies
pub type GlyphRangeDependencies = BTreeMap<FontStack, HashSet<GlyphRange>>;
