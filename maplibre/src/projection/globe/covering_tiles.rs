//! Visible-tile traversal for the vertical-perspective globe.

use std::collections::HashSet;

use cgmath::{InnerSpace, Vector4};
use thiserror::Error;

use super::{
    camera::GlobeCameraState,
    covering::{
        globe_tile_bounding_volume, GlobeTileBoundingVolume, GlobeTileBoundsError,
        TileElevationRange,
    },
};
use crate::coords::{LatLon, TileCoords, WorldTileCoords, ZoomLevel, MAX_ZOOM};

mod frustum;
mod lod;

use frustum::GlobeFrustum;

const ASSUMED_MAX_FEATURE_HEIGHT_METERS: f64 = 500.0;
const MAX_MERCATOR_HORIZON_DEGREES: f64 = 89.25;
const TILE_CULLING_HORIZON_ONSET_DEGREES: f64 = 15.0;

/// Inputs controlling fixed-level globe tile selection.
#[derive(Clone, Copy, Debug)]
pub struct GlobeCoveringOptions {
    /// Canonical zoom level to select.
    pub zoom: ZoomLevel,
    /// Fractional map zoom used by the per-tile LOD calculation.
    pub requested_zoom: f64,
    /// Enables per-tile zoom variation at high map zooms.
    pub variable_zoom: bool,
    /// Number of canonical neighbors to add around visible tiles.
    pub padding: i32,
    /// Maximum number of returned tiles after padding.
    pub max_tiles: usize,
    /// Elevation range included in culling volumes.
    pub elevation: TileElevationRange,
}

/// Failure while selecting visible globe tiles.
#[derive(Clone, Copy, Debug, Error, PartialEq)]
pub enum GlobeCoveringError {
    /// The coordinate model only supports canonical zoom levels through 31.
    #[error("globe covering zoom {zoom} exceeds the supported maximum")]
    UnsupportedZoom {
        /// Unsupported zoom level.
        zoom: u8,
    },
    /// A tile bounding volume could not be constructed.
    #[error("failed to construct globe tile bounds")]
    TileBounds {
        /// Underlying bounds error.
        #[source]
        source: GlobeTileBoundsError,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Intersection {
    None,
    Partial,
    Full,
}

#[derive(Clone, Copy)]
struct StackEntry {
    tile: TileCoords,
    fully_visible: bool,
}

/// Selects canonical tiles intersecting both the camera frustum and visible globe hemisphere.
pub fn covering_tiles(
    camera: &GlobeCameraState,
    options: GlobeCoveringOptions,
) -> Result<Vec<WorldTileCoords>, GlobeCoveringError> {
    if usize::from(u8::from(options.zoom)) >= MAX_ZOOM {
        return Err(GlobeCoveringError::UnsupportedZoom {
            zoom: u8::from(options.zoom),
        });
    }
    let mut stack = vec![StackEntry {
        tile: TileCoords::from((0, 0, ZoomLevel::new(0))),
        fully_visible: false,
    }];
    let mut visible = Vec::new();
    let lod = lod::LodContext::new(camera, options.requested_zoom);
    let frustum = GlobeFrustum::from_camera(camera);

    while let Some(entry) = stack.pop() {
        let bounds = globe_tile_bounding_volume(entry.tile, options.elevation)
            .map_err(|source| GlobeCoveringError::TileBounds { source })?;
        let intersection = if entry.fully_visible {
            Intersection::Full
        } else {
            tile_intersection(&frustum, camera.clipping_plane(), &bounds)
        };
        if intersection == Intersection::None {
            continue;
        }
        let target_zoom = if options.variable_zoom {
            lod.zoom_for_tile(entry.tile)
        } else {
            options.zoom
        };
        if entry.tile.z >= target_zoom {
            visible.push(WorldTileCoords {
                x: entry.tile.x as i32,
                y: entry.tile.y as i32,
                z: entry.tile.z,
            });
            continue;
        }
        push_children(&mut stack, entry.tile, intersection == Intersection::Full);
    }

    sort_by_center(&mut visible, camera.center(), options.zoom);
    Ok(add_padding(visible, options))
}

/// Returns the conservative elevation used to retain features near the frustum horizon.
pub fn elevation_for_tile_culling(camera: &GlobeCameraState, center_elevation: f64) -> f64 {
    let bottom_edge_above_horizontal = MAX_MERCATOR_HORIZON_DEGREES
        - camera.pitch_degrees()
        - camera.field_of_view_degrees() * 0.5;
    let proximity = ((TILE_CULLING_HORIZON_ONSET_DEGREES - bottom_edge_above_horizontal)
        / TILE_CULLING_HORIZON_ONSET_DEGREES)
        .clamp(0.0, 1.0);
    center_elevation + proximity * ASSUMED_MAX_FEATURE_HEIGHT_METERS
}

fn tile_intersection(
    frustum: &GlobeFrustum,
    clipping_plane: Vector4<f64>,
    bounds: &GlobeTileBoundingVolume,
) -> Intersection {
    let result = frustum.intersects(bounds);
    if result == Intersection::None {
        return result;
    }
    combine_intersections(
        result,
        classify_points(&bounds.points, |point| {
            clipping_plane.dot(point.extend(1.0))
        }),
    )
}

fn classify_points<T>(points: &[T], distance: impl Fn(&T) -> f64) -> Intersection {
    let inside = points.iter().filter(|point| distance(point) >= 0.0).count();
    if inside == 0 {
        Intersection::None
    } else if inside == points.len() {
        Intersection::Full
    } else {
        Intersection::Partial
    }
}

fn combine_intersections(left: Intersection, right: Intersection) -> Intersection {
    match (left, right) {
        (Intersection::None, _) | (_, Intersection::None) => Intersection::None,
        (Intersection::Full, Intersection::Full) => Intersection::Full,
        _ => Intersection::Partial,
    }
}

fn push_children(stack: &mut Vec<StackEntry>, tile: TileCoords, fully_visible: bool) {
    let child_zoom = ZoomLevel::new(u8::from(tile.z).saturating_add(1));
    for index in 0..4 {
        stack.push(StackEntry {
            tile: TileCoords::from((tile.x * 2 + index % 2, tile.y * 2 + index / 2, child_zoom)),
            fully_visible,
        });
    }
}

fn sort_by_center(tiles: &mut [WorldTileCoords], center: LatLon, nominal_zoom: ZoomLevel) {
    let center_x = center.longitude / 360.0 + 0.5;
    let latitude = center.latitude.to_radians();
    let center_y = (1.0 - latitude.tan().asinh() / std::f64::consts::PI) * 0.5;
    tiles.sort_by(|left, right| {
        distance_squared(*left, center_x, center_y, nominal_zoom).total_cmp(&distance_squared(
            *right,
            center_x,
            center_y,
            nominal_zoom,
        ))
    });
}

fn distance_squared(
    tile: WorldTileCoords,
    center_x: f64,
    center_y: f64,
    nominal_zoom: ZoomLevel,
) -> f64 {
    let count = 2_f64.powi(i32::from(u8::from(nominal_zoom)));
    let dx = center_x * count - 0.5 - f64::from(tile.x);
    let dy = center_y * count - 0.5 - f64::from(tile.y);
    dx * dx + dy * dy
}

fn add_padding(
    visible: Vec<WorldTileCoords>,
    options: GlobeCoveringOptions,
) -> Vec<WorldTileCoords> {
    if options.max_tiles == 0 {
        return Vec::new();
    }
    if options.padding <= 0 {
        return visible.into_iter().take(options.max_tiles).collect();
    }
    let mut seen = HashSet::new();
    let mut padded = Vec::new();
    for tile in visible {
        let count = 1_i64 << u8::from(tile.z);
        for delta_x in -options.padding..=options.padding {
            for delta_y in -options.padding..=options.padding {
                let y = i64::from(tile.y) + i64::from(delta_y);
                if !(0..count).contains(&y) {
                    continue;
                }
                let candidate = WorldTileCoords {
                    x: (i64::from(tile.x) + i64::from(delta_x)).rem_euclid(count) as i32,
                    y: y as i32,
                    z: tile.z,
                };
                if seen.insert(candidate) {
                    padded.push(candidate);
                    if padded.len() == options.max_tiles {
                        return padded;
                    }
                }
            }
        }
    }
    padded
}

#[cfg(test)]
mod tests;
