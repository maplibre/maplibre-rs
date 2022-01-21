//! File which exposes all kinds of coordinates used throughout mapr

use std::fmt;

/// Every tile has tile coordinates. These tile coordinates are also called
/// [Slippy map tilenames](https://wiki.openstreetmap.org/wiki/Slippy_map_tilenames).
///
/// # Coordinate System Origin
///
/// For Web Mercator the origin of the coordinate system is in the upper-left corner.
#[derive(Clone, Copy, Debug)]
pub struct TileCoords {
    pub x: u32,
    pub y: u32,
    pub z: u8,
}

impl TileCoords {
    /// Transforms the tile coordinates as defined by the tile grid into a representation which is
    /// used in the 3d-world.
    pub fn into_world_tile(self) -> WorldTileCoords {
        WorldTileCoords {
            x: self.x as i32 - crate::example::MUNICH_X as i32,
            y: self.y as i32 - crate::example::MUNICH_Y as i32,
            z: 0,
        }
    }
}

impl fmt::Display for TileCoords {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "T({x}, {y}, {z})", x = self.x, y = self.y, z = self.z)
    }
}

impl From<(u32, u32, u8)> for TileCoords {
    fn from(tuple: (u32, u32, u8)) -> Self {
        TileCoords {
            x: tuple.0,
            y: tuple.1,
            z: tuple.2,
        }
    }
}

/// Every tile has tile coordinates. Every tile coordinate can be mapped to a coordinate within
/// the world. This provides the freedom to map from [TMS](https://wiki.openstreetmap.org/wiki/TMS)
/// to [Slippy_map_tilenames](https://wiki.openstreetmap.org/wiki/Slippy_map_tilenames).
///
/// # Coordinate System Origin
///
/// The origin of the coordinate system is in the upper-left corner.
#[derive(Clone, Copy, Debug)]
pub struct WorldTileCoords {
    pub x: i32,
    pub y: i32,
    pub z: u8,
}

impl WorldTileCoords {
    pub fn into_world(self, extent: f32) -> WorldCoords {
        WorldCoords {
            x: self.x as f32 * extent,
            y: self.y as f32 * extent,
            z: self.z as f32,
        }
    }

    pub fn into_aligned(self) -> AlignedWorldTileCoords {
        AlignedWorldTileCoords(WorldTileCoords {
            x: div_floor(self.x, 2) * 2,
            y: div_floor(self.y, 2) * 2,
            z: self.z,
        })
    }
}

impl fmt::Display for WorldTileCoords {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WT({x}, {y}, {z})", x = self.x, y = self.y, z = self.z)
    }
}

impl From<(i32, i32, u8)> for WorldTileCoords {
    fn from(tuple: (i32, i32, u8)) -> Self {
        WorldTileCoords {
            x: tuple.0,
            y: tuple.1,
            z: tuple.2,
        }
    }
}

/// An aligned world tile coordinate aligns a world coordinate at a 4x4 tile raster within the
/// world. The aligned coordinates is defined by the coordinates of the upper left tile in the 4x4
/// tile raster divided by 2 and rounding to the ceiling.
///
///
/// # Coordinate System Origin
///
/// The origin of the coordinate system is in the upper-left corner.
pub struct AlignedWorldTileCoords(pub WorldTileCoords);

impl AlignedWorldTileCoords {
    pub fn into_upper_left(self) -> WorldTileCoords {
        self.0
    }

    pub fn upper_right(&self) -> WorldTileCoords {
        WorldTileCoords {
            x: self.0.x + 1,
            y: self.0.y,
            z: self.0.z,
        }
    }

    pub fn lower_left(&self) -> WorldTileCoords {
        WorldTileCoords {
            x: self.0.x,
            y: self.0.y - 1,
            z: self.0.z,
        }
    }

    pub fn lower_right(&self) -> WorldTileCoords {
        WorldTileCoords {
            x: self.0.x + 1,
            y: self.0.y + 1,
            z: self.0.z,
        }
    }
}

/// Actual coordinates within the 3d world.
///
/// # Coordinate System Origin
///
/// The origin of the coordinate system is in the upper-left corner.
#[derive(Clone, Copy, Debug)]
pub struct WorldCoords {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl fmt::Display for WorldCoords {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "W({x}, {y}, {z})", x = self.x, y = self.y, z = self.z)
    }
}

impl From<(f32, f32, f32)> for WorldCoords {
    fn from(tuple: (f32, f32, f32)) -> Self {
        WorldCoords {
            x: tuple.0,
            y: tuple.1,
            z: tuple.2,
        }
    }
}
