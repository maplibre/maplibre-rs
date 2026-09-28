//! Globe scale solving used by camera-for-bounds calculations.

use cgmath::{InnerSpace, Matrix4, Vector3, Vector4};

/// Clip-space dimension constrained by a bounds edge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClipDimension {
    /// Horizontal clip-space coordinate.
    X,
    /// Vertical clip-space coordinate.
    Y,
}

/// Solves the globe scale that projects `surface` onto one clip-space edge.
///
/// `center_surface` accounts for the camera moving toward the geographic center while zooming.
pub fn solve_vector_scale(
    surface: Vector3<f64>,
    center_surface: Vector3<f64>,
    projection: Matrix4<f64>,
    dimension: ClipDimension,
    target: f64,
) -> Option<f64> {
    let axis = match dimension {
        ClipDimension::X => Vector4::new(
            projection.x.x,
            projection.y.x,
            projection.z.x,
            projection.w.x,
        ),
        ClipDimension::Y => Vector4::new(
            projection.x.y,
            projection.y.y,
            projection.z.y,
            projection.w.y,
        ),
    };
    let homogeneous = Vector4::new(
        projection.x.w,
        projection.y.w,
        projection.z.w,
        projection.w.w,
    );
    let surface_axis = surface.dot(axis.truncate());
    let surface_w = surface.dot(homogeneous.truncate());
    let center_axis = center_surface.dot(axis.truncate());
    let center_w = center_surface.dot(homogeneous.truncate());
    let numerator = center_axis + axis.w - target * center_w - target * homogeneous.w;
    let denominator = center_axis - surface_axis - target * center_w + target * surface_w;
    let invalid_first = center_axis + target * surface_w == surface_axis + target * center_w;
    let invalid_second = homogeneous.w * (surface_axis - center_axis)
        + axis.w * (center_w - surface_w)
        + surface_axis * center_w
        == center_axis * surface_w;
    if invalid_first || invalid_second || denominator == 0.0 {
        return None;
    }
    let scale = numerator / denominator;
    scale.is_finite().then_some(scale)
}

/// Keeps the smaller non-negative candidate scale.
pub fn lesser_non_negative(current: f64, candidate: Option<f64>) -> f64 {
    match candidate {
        Some(candidate) if candidate >= 0.0 && candidate < current => candidate,
        _ => current,
    }
}

#[cfg(test)]
mod tests;
