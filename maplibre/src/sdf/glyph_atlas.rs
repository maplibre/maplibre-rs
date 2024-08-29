use std::collections::BTreeMap;

use crate::{
    euclid::Rect,
    sdf::{
        font_stack::FontStackHash,
        glyph::{GlyphID, GlyphMap, GlyphMetrics},
        TileSpace,
    },
};

// TODO structs
pub struct AlphaImage;

#[derive(Clone, Copy, Default)]
pub struct GlyphPosition {
    pub rect: Rect<u16, TileSpace>,
    pub metrics: GlyphMetrics,
}

pub type GlyphPositionMap = BTreeMap<GlyphID, GlyphPosition>;
pub type GlyphPositions = BTreeMap<FontStackHash, GlyphPositionMap>;

pub struct GlyphAtlas {
    pub image: AlphaImage,
    pub positions: GlyphPositions,
}

pub fn makeGlyphAtlas(glyphs: &GlyphMap) -> GlyphAtlas {
    todo!() // Shelfpack library!
}
