//! Globe interaction rotations shared by input frontends.

use std::f64::consts::PI;

use cgmath::{InnerSpace, Point2, Quaternion, Vector2};

use super::{
    camera::GlobeCameraState, globe_zoom_adjustment, lat_lon_bearing_from_orientation,
    lat_lon_to_unit_sphere, orientation_from_lat_lon_bearing, unit_sphere_to_lat_lon,
    wrap_longitude,
};
use crate::coords::LatLon;

pub mod animation;
pub mod bounds;
pub mod zoom;

const MIN_ROTATION_AXIS_SQUARED: f64 = 1e-24;
const MAX_INERTIA_LONGITUDE_DELTA: f64 = 179.5;
const MAX_VALID_LATITUDE: f64 = 85.051_128_779_806_59;
const PAN_FALLOFF_BAND: f64 = 0.1;
const PAN_MAX_ANGLE: f64 = PI * 0.98;
const DIAL_MIN_RADIUS_PIXELS: f64 = 20.0;

/// Camera-center change that keeps a geographic anchor under the cursor.
#[derive(Clone, Copy, Debug)]
pub struct GlobePanUpdate {
    /// Rotated geographic map center.
    pub center: LatLon,
    /// Zoom delta preserving the globe's apparent radius across latitude changes.
    pub zoom_adjustment: f64,
}

/// Limits an inertial pan target so an ease animation cannot choose the opposite direction.
pub fn clamp_pan_inertia_center(current_center: LatLon, target_center: LatLon) -> LatLon {
    let longitude_delta = target_center.longitude - current_center.longitude;
    if longitude_delta.abs() <= 180.0 {
        return target_center;
    }
    LatLon::new(
        target_center.latitude,
        current_center.longitude + longitude_delta.signum() * MAX_INERTIA_LONGITUDE_DELTA,
    )
}

/// Maps a pointer ray to the virtual globe trackball used during drag panning.
pub fn pan_surface_location(camera: &GlobeCameraState, pixel: Point2<f64>) -> Option<LatLon> {
    let origin = camera.camera_position();
    let distance = origin.magnitude();
    if distance <= 1.0 {
        return camera.screen_point_to_location(pixel);
    }
    let up = origin / distance;
    let direction = camera.ray_direction_from_pixel(pixel)?;
    let forward = -direction.dot(up);
    let lateral = direction + up * forward;
    let lateral_length = lateral.magnitude();
    if lateral_length < 1e-9 {
        return camera.screen_point_to_location(pixel);
    }
    let angle = lateral_length.atan2(forward);
    let horizon_angle = (1.0 / distance).asin();
    let handover_angle = horizon_angle * (1.0 - PAN_FALLOFF_BAND);
    if angle < handover_angle {
        return camera.screen_point_to_location(pixel);
    }
    let sin_handover = distance * handover_angle.sin();
    let target_at_handover = sin_handover.clamp(-1.0, 1.0).asin() - handover_angle;
    let slope_at_handover = distance * handover_angle.cos()
        / (1.0 - sin_handover * sin_handover).max(1e-12).sqrt()
        - 1.0;
    let room = PAN_MAX_ANGLE - target_at_handover;
    let excess = angle - handover_angle;
    let target = target_at_handover
        + room * (slope_at_handover * excess) / (room + slope_at_handover * excess);
    let target = target.clamp(0.0, PAN_MAX_ANGLE);
    let surface = up * target.cos() + lateral / lateral_length * target.sin();
    Some(unit_sphere_to_lat_lon(surface.normalize()))
}

/// Applies one pixel drag using GL JS's surface fallback and pole-dial behavior.
pub fn pan_camera_by_pixels(
    camera: &GlobeCameraState,
    requested_anchor: Point2<f64>,
    center_pixel: Point2<f64>,
    pan_delta: Vector2<f64>,
) -> Option<GlobePanUpdate> {
    let anchor = if camera.is_point_on_map_surface(requested_anchor) {
        requested_anchor
    } else {
        center_pixel
    };
    let current = pan_surface_location(camera, anchor)?;
    let target = pan_surface_location(camera, anchor - pan_delta)?;
    let mut update =
        pan_center_to_anchor(camera.center(), camera.bearing_degrees(), target, current);
    let pole_latitude = if camera.center().latitude >= 0.0 {
        90.0
    } else {
        -90.0
    };
    let pole = LatLon::new(pole_latitude, 0.0);
    let pole_pixel = camera.location_to_screen(pole, 0.0);
    update.center.longitude = fixed_bearing_longitude(
        camera.center(),
        anchor,
        pan_delta,
        pole_pixel,
        update.center.longitude,
    );
    update.zoom_adjustment =
        globe_zoom_adjustment(camera.center().latitude, update.center.latitude);
    Some(update)
}

fn fixed_bearing_longitude(
    old_center: LatLon,
    cursor: Point2<f64>,
    pan_delta: Vector2<f64>,
    pole_pixel: Point2<f64>,
    swing_longitude: f64,
) -> f64 {
    let radial = cursor - pole_pixel;
    let radius_squared = radial.magnitude2();
    let ramp = (1.0 - (MAX_VALID_LATITUDE - old_center.latitude.abs()) / 12.0).clamp(0.0, 1.0);
    let dial = ramp * ramp * (3.0 - 2.0 * ramp);
    let swing = wrap_longitude(swing_longitude - old_center.longitude);
    let sweep = (radial.x * pan_delta.y - radial.y * pan_delta.x)
        / radius_squared.max(DIAL_MIN_RADIUS_PIXELS * DIAL_MIN_RADIUS_PIXELS);
    let pole_sign = if old_center.latitude >= 0.0 {
        1.0
    } else {
        -1.0
    };
    let dial_longitude = pole_sign * sweep.to_degrees();
    wrap_longitude(old_center.longitude + (1.0 - dial) * swing + dial * dial_longitude)
}

/// Rotates a globe so `anchor` moves to the surface location currently under the cursor.
///
/// The rotation uses the same surface-vector and quaternion frames as MapLibre GL JS. Bearing is
/// intentionally fixed for drag panning.
pub fn pan_center_to_anchor(
    current_center: LatLon,
    bearing_degrees: f64,
    anchor: LatLon,
    cursor_location: LatLon,
) -> GlobePanUpdate {
    let target = lat_lon_to_unit_sphere(anchor);
    let current = lat_lon_to_unit_sphere(cursor_location);
    let axis = target.cross(current);
    let axis_length_squared = axis.magnitude2();
    let dot = target.dot(current).clamp(-1.0, 1.0);
    let half_angle = dot.acos() * 0.5;
    let delta = if axis_length_squared > MIN_ROTATION_AXIS_SQUARED {
        let scaled_axis = axis * (half_angle.sin() / axis_length_squared.sqrt());
        Quaternion::new(
            half_angle.cos(),
            scaled_axis.y,
            -scaled_axis.x,
            scaled_axis.z,
        )
    } else {
        Quaternion::new(1.0, 0.0, 0.0, 0.0)
    };
    let orientation = orientation_from_lat_lon_bearing(current_center, bearing_degrees) * delta;
    let rotated = lat_lon_bearing_from_orientation(orientation);
    let center = LatLon::new(
        rotated.center.latitude.clamp(-90.0, 90.0),
        wrap_longitude(rotated.center.longitude),
    );

    GlobePanUpdate {
        center,
        zoom_adjustment: globe_zoom_adjustment(current_center.latitude, center.latitude),
    }
}

#[cfg(test)]
mod tests;
