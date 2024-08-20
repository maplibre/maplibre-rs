// Except for to-do comments this file was fully translated

use crate::sdf::bidi::Char16;
use crate::sdf::font_stack::{FontStack, FontStackHash};
use crate::sdf::glyph_range::GlyphRange;
use bitflags::bitflags;
use geo_types::Rect;
use std::collections::{HashMap, HashSet};

// TODO structs
struct AlphaImage;

pub type GlyphID = Char16;
pub type GlyphIDs = HashSet<GlyphID>;

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

#[derive(Default)]
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
