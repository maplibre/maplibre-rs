//! Projection uniforms shared by Mercator, globe, terrain, and custom-layer rendering.

use cgmath::{InnerSpace, Matrix4, Vector3, Vector4};
use thiserror::Error;

use crate::coords::{LatLon, TileCoords, EXTENT};

/// Matrices used while selecting or transitioning between map projections.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectionMatrices {
    /// Projects Mercator tile coordinates to clip space.
    pub mercator: Matrix4<f32>,
    /// Projects the unit globe to clip space.
    pub globe: Matrix4<f32>,
}

/// Inputs controlling how projection data is prepared for one draw.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ProjectionDataParams {
    /// Canonical tile rendered by this draw, or `None` for world-space draws.
    pub tile: Option<TileCoords>,
    /// Requests a pixel-aligned matrix where the active projection supports it.
    pub aligned: bool,
    /// Requests terrain transforms for this draw.
    pub apply_terrain_matrix: bool,
    /// Enables globe projection or its transition for this draw.
    pub apply_globe_matrix: bool,
}

/// Projection data consumed by renderer shaders.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RendererProjectionData {
    /// Active main projection matrix.
    pub main_matrix: Matrix4<f32>,
    /// Mercator tile offset and per-tile-coordinate scale as `[x, y, scale_x, scale_y]`.
    pub tile_mercator_coords: Vector4<f32>,
    /// Unit-sphere horizon plane; the visible half-space has non-negative distance.
    pub clipping_plane: Vector4<f32>,
    /// Interpolation from Mercator zero to globe one.
    pub projection_transition: f32,
    /// Mercator projection matrix used during globe transitions.
    pub fallback_matrix: Matrix4<f32>,
    /// Whether zoom-zero line fragments must be clipped at the antimeridian.
    pub clip_antimeridian: bool,
}

/// Camera geometry needed to derive the globe horizon plane.
#[derive(Clone, Copy, Debug)]
pub struct GlobeViewGeometry {
    /// Geographic location at the center of the viewport.
    pub center: LatLon,
    /// Clockwise map bearing in degrees.
    pub bearing_degrees: f64,
    /// Camera pitch in degrees.
    pub pitch_degrees: f64,
    /// Camera distance to the map center in pixels.
    pub camera_to_center_distance: f64,
    /// Globe radius in pixels at the map center latitude.
    pub globe_radius_pixels: f64,
}

/// Failure while preparing renderer projection data.
#[derive(Clone, Copy, Debug, Error, PartialEq)]
pub enum ProjectionDataError {
    /// Globe radius must be finite and greater than zero.
    #[error("globe radius must be finite and positive, got {radius}")]
    InvalidGlobeRadius {
        /// Invalid radius in pixels.
        radius: f64,
    },
    /// Camera-to-center distance must be finite and non-negative.
    #[error("camera-to-center distance must be finite and non-negative, got {distance}")]
    InvalidCameraDistance {
        /// Invalid distance in pixels.
        distance: f64,
    },
    /// An angle required for globe orientation must be finite.
    #[error("{name} must be finite, got {degrees}")]
    InvalidAngle {
        /// Name of the invalid view angle.
        name: &'static str,
        /// Invalid value in degrees.
        degrees: f64,
    },
}

/// Combines projection matrices and per-draw parameters into renderer data.
pub fn compose_projection_data(
    matrices: ProjectionMatrices,
    clipping_plane: Vector4<f32>,
    globe_transition: f32,
    params: ProjectionDataParams,
) -> RendererProjectionData {
    let transition = if globe_transition.is_finite() {
        globe_transition.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let use_globe_rendering = transition > 0.0;
    RendererProjectionData {
        main_matrix: if use_globe_rendering {
            matrices.globe
        } else {
            matrices.mercator
        },
        tile_mercator_coords: tile_mercator_coordinates(params.tile),
        clipping_plane,
        projection_transition: if params.apply_globe_matrix {
            transition
        } else {
            0.0
        },
        fallback_matrix: matrices.mercator,
        clip_antimeridian: params.tile.is_some_and(|tile| u8::from(tile.z) == 0),
    }
}

/// Returns Mercator offset and scale for tile-local coordinates in range `0..EXTENT`.
pub fn tile_mercator_coordinates(tile: Option<TileCoords>) -> Vector4<f32> {
    let Some(tile) = tile else {
        let scale = (1.0 / EXTENT) as f32;
        return Vector4::new(0.0, 0.0, scale, scale);
    };
    let tile_count = 2_f64.powi(i32::from(u8::from(tile.z)));
    let extent_scale = 1.0 / (tile_count * EXTENT);
    Vector4::new(
        (f64::from(tile.x) / tile_count) as f32,
        (f64::from(tile.y) / tile_count) as f32,
        extent_scale as f32,
        extent_scale as f32,
    )
}

/// Computes the unit-sphere horizon plane used to reject occluded globe geometry.
pub fn compute_globe_clipping_plane(
    geometry: GlobeViewGeometry,
) -> Result<Vector4<f64>, ProjectionDataError> {
    validate_view_geometry(geometry)?;
    let pitch = geometry.pitch_degrees.to_radians();
    let camera_to_surface = geometry.camera_to_center_distance / geometry.globe_radius_pixels;
    let camera_horizontal = pitch.sin() * camera_to_surface;
    let camera_vertical = pitch.cos() * camera_to_surface + 1.0;
    let camera_to_globe_center = camera_horizontal.hypot(camera_vertical);
    let tangent_plane_distance = 1.0 / camera_to_globe_center;

    let camera_direction = Vector3::new(-camera_horizontal, camera_vertical, 0.0).normalize();
    let plane_normal = rotate_globe_plane_normal(
        Vector3::new(0.0, camera_direction.x, camera_direction.y),
        geometry.center,
        geometry.bearing_degrees,
    )
    .normalize();
    Ok(plane_normal.extend(-tangent_plane_distance))
}

fn validate_view_geometry(geometry: GlobeViewGeometry) -> Result<(), ProjectionDataError> {
    if !geometry.globe_radius_pixels.is_finite() || geometry.globe_radius_pixels <= 0.0 {
        return Err(ProjectionDataError::InvalidGlobeRadius {
            radius: geometry.globe_radius_pixels,
        });
    }
    if !geometry.camera_to_center_distance.is_finite() || geometry.camera_to_center_distance < 0.0 {
        return Err(ProjectionDataError::InvalidCameraDistance {
            distance: geometry.camera_to_center_distance,
        });
    }
    for (name, degrees) in [
        ("center latitude", geometry.center.latitude),
        ("center longitude", geometry.center.longitude),
        ("bearing", geometry.bearing_degrees),
        ("pitch", geometry.pitch_degrees),
    ] {
        if !degrees.is_finite() {
            return Err(ProjectionDataError::InvalidAngle { name, degrees });
        }
    }
    Ok(())
}

fn rotate_globe_plane_normal(
    normal: Vector3<f64>,
    center: LatLon,
    bearing_degrees: f64,
) -> Vector3<f64> {
    let bearing = bearing_degrees.to_radians();
    let latitude = center.latitude.to_radians();
    let longitude = center.longitude.to_radians();
    let after_bearing = Vector3::new(
        normal.x * bearing.cos() + normal.y * bearing.sin(),
        -normal.x * bearing.sin() + normal.y * bearing.cos(),
        normal.z,
    );
    let after_latitude = Vector3::new(
        after_bearing.x,
        after_bearing.y * latitude.cos() + after_bearing.z * latitude.sin(),
        -after_bearing.y * latitude.sin() + after_bearing.z * latitude.cos(),
    );
    Vector3::new(
        after_latitude.x * longitude.cos() + after_latitude.z * longitude.sin(),
        after_latitude.y,
        -after_latitude.x * longitude.sin() + after_latitude.z * longitude.cos(),
    )
}

#[cfg(test)]
mod tests;
