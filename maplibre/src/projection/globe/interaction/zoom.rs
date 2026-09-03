//! Pointer-centered zoom blending for globe controls.

use cgmath::{InnerSpace, Vector3};

use crate::{
    coords::LatLon,
    projection::globe::{globe_zoom_adjustment, wrap_longitude},
};

const MAX_VALID_LATITUDE: f64 = 85.051_128_779_806_59;
const RAY_SURFACE_DISTANCE_FOR_SLOWING_START: f64 = 0.3;
const SLOWING_MULTIPLIER: f64 = 0.5;
const INTERPOLATE_TO_HEURISTIC_START_LNG: f64 = 45.0;
const INTERPOLATE_TO_HEURISTIC_END_LNG: f64 = 85.0;
const INTERPOLATE_TO_HEURISTIC_EXPONENT: f64 = 0.25;
const INTERPOLATE_TO_HEURISTIC_START_HORIZON: f64 = 0.95;
const INTERPOLATE_TO_HEURISTIC_END_HORIZON: f64 = 0.999;
const SLOWING_RADIUS_START: f64 = 0.9;
const SLOWING_RADIUS_STOP: f64 = 0.5;
const SLOWING_RADIUS_SLOW_FACTOR: f64 = 0.25;

/// Inputs captured after applying one zoom delta and its exact anchor correction.
#[derive(Clone, Copy, Debug)]
pub struct GlobeZoomInput {
    /// Center before the zoom delta.
    pub start_center: LatLon,
    /// Zoom after applying the requested, constrained delta.
    pub zoom_after_delta: f64,
    /// Actual constrained zoom delta.
    pub zoom_delta: f64,
    /// Geographic location under the pointer before zooming.
    pub pointer_location: LatLon,
    /// Center produced by exact set-location-at-point anchoring.
    pub exact_center: LatLon,
    /// Camera origin in unit-globe coordinates.
    pub ray_origin: Vector3<f64>,
    /// Normalized pointer ray direction in unit-globe coordinates.
    pub ray_direction: Vector3<f64>,
    /// Globe radius divided by the smaller viewport dimension.
    pub relative_globe_radius: f64,
}

/// Center and zoom after horizon-safe pointer zoom blending.
#[derive(Clone, Copy, Debug)]
pub struct GlobeZoomUpdate {
    /// Blended geographic center.
    pub center: LatLon,
    /// Latitude-compensated zoom.
    pub zoom: f64,
}

/// Blends exact pointer anchoring with GL JS's stable horizon heuristic.
pub fn zoom_around_globe(input: GlobeZoomInput) -> GlobeZoomUpdate {
    if input.zoom_delta == 0.0 {
        return GlobeZoomUpdate {
            center: input.start_center,
            zoom: input.zoom_after_delta,
        };
    }
    let raw_longitude_delta = shortest_angle_delta(
        input.start_center.longitude,
        input.pointer_location.longitude,
    );
    let longitude_delta = raw_longitude_delta / (raw_longitude_delta.abs() / 180.0 + 1.0);
    let latitude_delta =
        shortest_angle_delta(input.start_center.latitude, input.pointer_location.latitude);
    let ray_distance = closest_ray_distance(input.ray_origin, input.ray_direction);
    let distance_from_surface = ray_distance - 1.0;
    let distance_factor = (-((distance_from_surface - RAY_SURFACE_DISTANCE_FOR_SLOWING_START)
        .max(0.0))
        * SLOWING_MULTIPLIER)
        .exp();
    let horizon_factor = remap_saturate(
        ray_distance,
        INTERPOLATE_TO_HEURISTIC_START_HORIZON,
        INTERPOLATE_TO_HEURISTIC_END_HORIZON,
        0.0,
        1.0,
    );
    let radius_factor = remap_saturate(
        input.relative_globe_radius,
        SLOWING_RADIUS_START,
        SLOWING_RADIUS_STOP,
        1.0,
        SLOWING_RADIUS_SLOW_FACTOR,
    );
    let slowing_factor = distance_factor.min(lerp(1.0, radius_factor, horizon_factor));
    let movement_factor = (1.0 - 2_f64.powf(-input.zoom_delta)) * slowing_factor;
    let heuristic_center = LatLon::new(
        (input.start_center.latitude + latitude_delta * movement_factor)
            .clamp(-MAX_VALID_LATITUDE, MAX_VALID_LATITUDE),
        input.start_center.longitude + longitude_delta * movement_factor,
    );
    let longitude_factor = remap_saturate(
        raw_longitude_delta.abs(),
        INTERPOLATE_TO_HEURISTIC_START_LNG,
        INTERPOLATE_TO_HEURISTIC_END_LNG,
        0.0,
        1.0,
    );
    let heuristic_factor = longitude_factor
        .max(horizon_factor)
        .powf(INTERPOLATE_TO_HEURISTIC_EXPONENT);
    let exact_to_heuristic_longitude =
        shortest_angle_delta(input.exact_center.longitude, heuristic_center.longitude);
    let exact_to_heuristic_latitude =
        shortest_angle_delta(input.exact_center.latitude, heuristic_center.latitude);
    let center = LatLon::new(
        input.exact_center.latitude + exact_to_heuristic_latitude * heuristic_factor,
        wrap_longitude(
            input.exact_center.longitude + exact_to_heuristic_longitude * heuristic_factor,
        ),
    );
    GlobeZoomUpdate {
        center,
        zoom: input.zoom_after_delta
            + globe_zoom_adjustment(input.start_center.latitude, center.latitude),
    }
}

fn closest_ray_distance(origin: Vector3<f64>, direction: Vector3<f64>) -> f64 {
    let distance_along_ray = -origin.dot(direction);
    (origin + direction * distance_along_ray).magnitude()
}

fn shortest_angle_delta(start: f64, end: f64) -> f64 {
    wrap_longitude(end - start)
}

fn remap_saturate(value: f64, input_start: f64, input_end: f64, low: f64, high: f64) -> f64 {
    let progress = ((value - input_start) / (input_end - input_start)).clamp(0.0, 1.0);
    lerp(low, high, progress)
}

fn lerp(start: f64, end: f64, progress: f64) -> f64 {
    start + (end - start) * progress
}

#[cfg(test)]
mod tests;
