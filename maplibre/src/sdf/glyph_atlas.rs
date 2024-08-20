use crate::sdf::font_stack::FontStackHash;
use crate::sdf::glyph::{GlyphID, GlyphMap, GlyphMetrics};
use geo_types::Rect;
use std::collections::HashMap;

// TODO structs
pub struct AlphaImage;

pub struct GlyphPosition {
    pub rect: Rect<u16>,
    pub metrics: GlyphMetrics,
}

pub type GlyphPositionMap = HashMap<GlyphID, GlyphPosition>;
pub type GlyphPositions = HashMap<FontStackHash, GlyphPositionMap>;

pub struct GlyphAtlas {
    pub image: AlphaImage,
    pub positions: GlyphPositions,
}

pub fn makeGlyphAtlas(glyphs: &GlyphMap) -> GlyphAtlas {
    todo!() // Shelfpack library!
}
