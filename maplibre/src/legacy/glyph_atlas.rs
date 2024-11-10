//! Translated from https://github.com/maplibre/maplibre-native/blob/4add9ea/src/mbgl/text/glyph_atlas.cpp

use std::collections::BTreeMap;

use crate::{
    euclid::Rect,
    legacy::{
        font_stack::FontStackHash,
        glyph::{GlyphID, GlyphMap, GlyphMetrics},
        TileSpace,
    },
};

// TODO structs
/// maplibre/maplibre-native#4add9ea original name: AlphaImage
pub struct AlphaImage;

/// maplibre/maplibre-native#4add9ea original name: GlyphPosition
#[derive(Clone, Copy, Default)]
pub struct GlyphPosition {
    pub rect: Rect<u16, TileSpace>,
    pub metrics: GlyphMetrics,
}

/// maplibre/maplibre-native#4add9ea original name: GlyphPositionMap
pub type GlyphPositionMap = BTreeMap<GlyphID, GlyphPosition>;
/// maplibre/maplibre-native#4add9ea original name: GlyphPositions
pub type GlyphPositions = BTreeMap<FontStackHash, GlyphPositionMap>;

/// maplibre/maplibre-native#4add9ea original name: GlyphAtlas
pub struct GlyphAtlas {
    pub image: AlphaImage,
    pub positions: GlyphPositions,
}

/// maplibre/maplibre-native#4add9ea original name: makeGlyphAtlas
pub fn makeGlyphAtlas(glyphs: &GlyphMap) -> GlyphAtlas {
    todo!() // Shelfpack library!
}
