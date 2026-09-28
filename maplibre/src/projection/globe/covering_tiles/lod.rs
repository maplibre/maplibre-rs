use cgmath::Point2;

use super::super::{camera::GlobeCameraState, covering::distance_to_tile_2d};
use crate::coords::{TileCoords, ZoomLevel, MAX_ZOOM};

const MAX_MERCATOR_HORIZON_DEGREES: f64 = 89.25;
const MAX_ZOOM_LEVELS_ON_SCREEN: f64 = 9.314;
const TILE_COUNT_MAX_MIN_RATIO: f64 = 3.0;
const INTEGRATION_POINTS: usize = 10;

pub(super) struct LodContext {
    camera: Point2<f64>,
    distance_z: f64,
    distance_to_center_3d: f64,
    requested_zoom: f64,
    field_of_view_degrees: f64,
}

impl LodContext {
    pub(super) fn new(camera: &GlobeCameraState, requested_zoom: f64) -> Self {
        let center = mercator_center(camera);
        let distance = camera.camera_to_center_distance() / camera.world_size();
        let pitch = camera.pitch_degrees().to_radians();
        let bearing = camera.bearing_degrees().to_radians();
        let horizontal = pitch.sin();
        let direction_x = horizontal * bearing.sin();
        let direction_y = -horizontal * bearing.cos();
        let camera_point = Point2::new(
            center.x - distance * direction_x,
            center.y - distance * direction_y,
        );
        let distance_z = distance * pitch.cos();
        let distance_to_center_2d = (center.x - camera_point.x).hypot(center.y - camera_point.y);
        Self {
            camera: camera_point,
            distance_z,
            distance_to_center_3d: distance_to_center_2d.hypot(distance_z),
            requested_zoom,
            field_of_view_degrees: camera.field_of_view_degrees(),
        }
    }

    pub(super) fn zoom_for_tile(&self, tile: TileCoords) -> ZoomLevel {
        let distance_2d = distance_to_tile_2d(self.camera, tile);
        let desired = calculate_tile_zoom(
            self.requested_zoom,
            distance_2d,
            self.distance_z,
            self.distance_to_center_3d,
            self.field_of_view_degrees,
        )
        .floor()
        .clamp(0.0, (MAX_ZOOM - 1) as f64);
        ZoomLevel::new(desired as u8)
    }
}

fn mercator_center(camera: &GlobeCameraState) -> Point2<f64> {
    let center = camera.center();
    let x = center.longitude / 360.0 + 0.5;
    let latitude = center.latitude.to_radians();
    let y = (1.0 - latitude.tan().asinh() / std::f64::consts::PI) * 0.5;
    Point2::new(x, y)
}

fn calculate_tile_zoom(
    requested_center_zoom: f64,
    distance_to_tile_2d: f64,
    distance_to_tile_z: f64,
    distance_to_center_3d: f64,
    field_of_view_degrees: f64,
) -> f64 {
    let pitch_behavior = pitch_tile_loading_behavior(field_of_view_degrees);
    let center_pitch = (distance_to_tile_z / distance_to_center_3d).acos();
    let half_fov = (field_of_view_degrees * 0.5).to_radians();
    let tile_count_pitch_zero = 2.0 * integral_cos_power(pitch_behavior - 1.0, 0.0, half_fov);
    let highest_pitch = (center_pitch + half_fov).min(MAX_MERCATOR_HORIZON_DEGREES.to_radians());
    let lowest_pitch = (center_pitch - half_fov).min(highest_pitch);
    let tile_count = integral_cos_power(pitch_behavior - 1.0, lowest_pitch, highest_pitch);
    let tile_pitch = (distance_to_tile_2d / distance_to_tile_z).atan();
    let distance_to_tile_3d = distance_to_tile_2d.hypot(distance_to_tile_z);
    let distance_scale = distance_to_center_3d / distance_to_tile_3d / 0.5_f64.max(half_fov.cos());
    requested_center_zoom + distance_scale.log2() + pitch_behavior * tile_pitch.cos().log2() * 0.5
        - (tile_count / tile_count_pitch_zero / TILE_COUNT_MAX_MIN_RATIO)
            .max(1.0)
            .log2()
            * 0.5
}

fn pitch_tile_loading_behavior(field_of_view_degrees: f64) -> f64 {
    let numerator = (MAX_MERCATOR_HORIZON_DEGREES - field_of_view_degrees)
        .to_radians()
        .cos();
    let denominator = MAX_MERCATOR_HORIZON_DEGREES.to_radians().cos();
    2.0 * ((MAX_ZOOM_LEVELS_ON_SCREEN - 1.0) / (numerator / denominator).log2() - 1.0)
}

fn integral_cos_power(power: f64, start: f64, end: f64) -> f64 {
    let width = (end - start) / INTEGRATION_POINTS as f64;
    (0..INTEGRATION_POINTS)
        .map(|index| {
            let x = start + (index as f64 + 0.5) * width;
            width * x.cos().powf(power)
        })
        .sum()
}
