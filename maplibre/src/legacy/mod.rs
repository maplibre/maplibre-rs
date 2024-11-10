pub mod bidi;
pub mod buckets;
pub mod collision_feature;
pub mod collision_index;
pub mod font_stack;
pub mod geometry;
pub mod geometry_tile_data;
pub mod glyph;
pub mod glyph_atlas;
pub mod glyph_range;
pub mod grid_index;
pub mod image;
pub mod image_atlas;
pub mod layout;
pub mod quads;
pub mod shaping;
pub mod style_types;
pub mod tagged_string;
pub mod util;

// TODO where should this live?
pub struct TileSpace; // The unit in which geometries or symbols are on a tile (0-EXTENT)
pub struct ScreenSpace;

// TODO where should this live?
#[derive(Copy, Clone, PartialEq)]
pub enum MapMode {
    ///< continually updating map
    Continuous,
    ///< a once-off still image of an arbitrary viewport
    Static,
    ///< a once-off still image of a single tile
    Tile,
}

// TODO this is just a dummy
#[derive(Copy, Clone)]
pub struct CanonicalTileID {
    pub x: u32,
    pub y: u32,
    pub z: u8,
}

// TODO
#[derive(Copy, Clone)]
pub struct OverscaledTileID {
    pub canonical: CanonicalTileID,
    pub overscaledZ: u8,
}

impl OverscaledTileID {
    pub fn overscaleFactor(&self) -> u32 {
        return 1 << (self.overscaledZ - self.canonical.z);
    }
}
