// Except for to-do comments this file was fully translated

use bitflags::bitflags;
use geo_types::Rect;
use std::collections::{HashMap, HashSet};
use std::ops::Range;

// TODO
struct AlphaImage;
type GlyphRange = Range<u32>;
struct FontStackHash;
struct FontStack;

pub type GlyphID = char; // was char16_t
pub type GlyphIDs = HashSet<GlyphID>;

// Note: this only works for the BMP
pub fn getGlyphRange(glyph: GlyphID) -> GlyphRange {
    let mut start: u32 = (glyph as u32 / 256) * 256;
    let mut end = (start + 255);
    if (start > 65280) {
        start = 65280;
    }
    if (end > 65535) {
        end = 65535;
    }
    return start..end;
}

#[derive(PartialEq)]
pub struct GlyphMetrics {
    pub width: u32,
    pub height: u32,
    pub left: i32,
    pub top: i32,
    pub advance: u32,
}

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

pub type Glyphs = HashMap<GlyphID, Option<Glyph>>;
pub type GlyphMap = HashMap<FontStackHash, Glyphs>;

pub struct PositionedGlyph {
    pub glyph: GlyphID,
    pub x: f64,
    pub y: f64,
    pub vertical: bool,
    pub font: FontStackHash,
    pub scale: f64,
    pub rect: Rect<u16>,
    pub metrics: GlyphMetrics,
    pub imageID: Option<String>,
    // Maps positioned glyph to TaggedString section
    pub sectionIndex: usize,
}

pub struct PositionedLine {
    pub positionedGlyphs: Vec<PositionedGlyph>,
    pub lineOffset: f64,
}

pub struct Shaping {
    pub positionedLines: Vec<PositionedLine>,
    pub top: f64,
    pub bottom: f64,
    pub left: f64,
    pub right: f64,
    pub writingMode: WritingModeType,

    // The y offset *should* be part of the font metadata.
    pub verticalizable: bool,
    pub iconsInText: bool,
}
impl Shaping {
    pub const yOffset: i32 = -17;
}

impl Into<bool> for Shaping {
    fn into(self) -> bool {
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

pub type GlyphDependencies = HashMap<FontStack, GlyphIDs>;
pub type GlyphRangeDependencies = HashMap<FontStack, HashSet<GlyphRange>>;
