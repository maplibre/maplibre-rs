//! File which exposes all kinds of coordinates used throughout mapr

use crate::io::tile_cache::TileCache;
use crate::util::math::{div_away, div_floor, Aabb2};
use cgmath::num_traits::Pow;
use cgmath::{Matrix4, Point3, SquareMatrix, Vector3};
use std::fmt;
use std::ops::Mul;
use style_spec::source::TileAdressingScheme;

pub const EXTENT_UINT: u32 = 4096;
pub const EXTENT_SINT: i32 = EXTENT_UINT as i32;
pub const EXTENT: f64 = EXTENT_UINT as f64;

/// Every tile has tile coordinates. These tile coordinates are also called
/// [Slippy map tilenames](https://wiki.openstreetmap.org/wiki/Slippy_map_tilenames).
///
/// # Coordinate System Origin
///
/// For Web Mercator the origin of the coordinate system is in the upper-left corner.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct TileCoords {
    pub x: u32,
    pub y: u32,
    pub z: u8,
}

impl TileCoords {
    /// Transforms the tile coordinates as defined by the tile grid addressing scheme into a representation which is
    /// used in the 3d-world.
    pub fn into_world_tile(self, scheme: TileAdressingScheme) -> WorldTileCoords {
        match scheme {
            TileAdressingScheme::XYZ => WorldTileCoords {
                x: self.x as i32,
                y: self.y as i32,
                z: self.z,
            },
            TileAdressingScheme::TMS => WorldTileCoords {
                x: self.x as i32,
                y: (2u32.pow(self.z as u32) - 1 - self.y) as i32,
                z: self.z,
            },
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorldTileCoords {
    pub x: i32,
    pub y: i32,
    pub z: u8,
}

impl WorldTileCoords {
    pub fn into_tile(self, scheme: TileAdressingScheme) -> TileCoords {
        match scheme {
            TileAdressingScheme::XYZ => TileCoords {
                x: self.x as u32,
                y: self.y as u32,
                z: self.z,
            },
            TileAdressingScheme::TMS => TileCoords {
                x: self.x as u32,
                y: 2u32.pow(self.z as u32) - 1 - self.y as u32,
                z: self.z,
            },
        }
    }

    pub fn transform_for_zoom(&self, zoom: f64) -> Matrix4<f64> {
        /*
           For tile.z = zoom:
               => scale = 512
           If tile.z < zoom:
               => scale > 512
           If tile.z > zoom:
               => scale < 512
        */
        let tile_scale = TILE_SIZE * 2.0.pow(zoom - self.z as f64);

        let translate = Matrix4::from_translation(Vector3::new(
            self.x as f64 * tile_scale,
            self.y as f64 * tile_scale,
            0.0,
        ));

        // Divide by EXTENT to normalize tile
        let normalize = Matrix4::from_nonuniform_scale(1.0 / EXTENT, 1.0 / EXTENT, 1.0);
        // Scale tiles where the zoom level matches its z to 512x512
        let scale = Matrix4::from_nonuniform_scale(tile_scale, tile_scale, 1.0);
        translate * normalize * scale
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
    pub fn upper_left(self) -> WorldTileCoords {
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

/// Actual coordinates within the 3D world. The `z` value of the [`WorldCoors`] is not related to
/// the `z` value of the [`WorldTileCoors`]. In the 3D world all tiles are rendered at `z` values
/// which are determined only by the render engine and not by the zoom level.
///
/// # Coordinate System Origin
///
/// The origin of the coordinate system is in the upper-left corner.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldCoords {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

const TILE_SIZE: f64 = 512.0;

fn world_size_at_zoom(zoom: f64) -> f64 {
    TILE_SIZE * 2.0.pow(zoom)
}

fn tiles_with_z(z: u8) -> f64 {
    2.0.pow(z)
}

impl WorldCoords {
    pub fn at_ground(x: f64, y: f64) -> Self {
        Self { x, y, z: 0.0 }
    }

    pub fn into_world_tile(self, z: u8, zoom: f64) -> WorldTileCoords {
        let tile_scale = 2.0.pow(z as f64 - zoom) / TILE_SIZE;
        let x = self.x * tile_scale;
        let y = self.y * tile_scale;

        WorldTileCoords {
            x: x as i32,
            y: y as i32,
            z,
        }
    }
}

impl fmt::Display for WorldCoords {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "W({x}, {y}, {z})", x = self.x, y = self.y, z = self.z)
    }
}

impl From<(f32, f32, f32)> for WorldCoords {
    fn from(tuple: (f32, f32, f32)) -> Self {
        WorldCoords {
            x: tuple.0 as f64,
            y: tuple.1 as f64,
            z: tuple.2 as f64,
        }
    }
}

impl From<(f64, f64, f64)> for WorldCoords {
    fn from(tuple: (f64, f64, f64)) -> Self {
        WorldCoords {
            x: tuple.0,
            y: tuple.1,
            z: tuple.2,
        }
    }
}

impl From<Point3<f64>> for WorldCoords {
    fn from(point: Point3<f64>) -> Self {
        WorldCoords {
            x: point.x,
            y: point.y,
            z: point.z,
        }
    }
}

pub struct ViewRegion {
    min_tile: WorldTileCoords,
    max_tile: WorldTileCoords,
    z: u8,
}

impl ViewRegion {
    pub fn new(view_region: Aabb2<f64>, zoom: f64, z: u8) -> Self {
        let min_world: WorldCoords = WorldCoords::at_ground(view_region.min.x, view_region.min.y);
        let min_world_tile: WorldTileCoords = min_world.into_world_tile(z, zoom);
        let max_world: WorldCoords = WorldCoords::at_ground(view_region.max.x, view_region.max.y);
        let max_world_tile: WorldTileCoords = max_world.into_world_tile(z, zoom);

        Self {
            min_tile: min_world_tile,
            max_tile: max_world_tile,
            z,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = WorldTileCoords> + '_ {
        (self.min_tile.x..self.max_tile.x + 1).flat_map(move |x| {
            (self.min_tile.y..self.max_tile.y + 1).map(move |y| {
                let tile_coord: WorldTileCoords = (x, y, self.z as u8).into();
                tile_coord
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::coords::{ViewRegion, WorldCoords, WorldTileCoords, EXTENT};
    use crate::util::math::Aabb2;
    use cgmath::{Point2, Vector3, Vector4, Zero};

    const TOP_LEFT: Vector4<f64> = Vector4::new(0.0, 0.0, 0.0, 1.0);
    const BOTTOM_RIGHT: Vector4<f64> = Vector4::new(EXTENT, EXTENT, 0.0, 1.0);

    fn to_from_world(tile: (i32, i32, u8), zoom: f64) {
        let tile = WorldTileCoords::from(tile);
        let p1 = tile.transform_for_zoom(zoom) * TOP_LEFT;
        let p2 = tile.transform_for_zoom(zoom) * BOTTOM_RIGHT;
        println!("{:?}\n{:?}", p1, p2);

        assert_eq!(
            WorldCoords::from((p1.x, p1.y, 0.0)).into_world_tile(zoom.floor() as u8, zoom),
            tile
        );
    }

    #[test]
    fn world_coords_tests() {
        to_from_world((1, 0, 1), 1.0);
        to_from_world((67, 42, 7), 7.0);
        to_from_world((17421, 11360, 15), 15.0);
    }

    #[test]
    fn test_view_region() {
        for tile_coords in ViewRegion::new(
            Aabb2::new(Point2::new(0.0, 0.0), Point2::new(2000.0, 2000.0)),
            0.0,
            0,
        )
        .iter()
        {
            println!("{}", tile_coords);
        }
    }
}
