#![allow(clippy::panic)]

use cgmath::{Point2, Vector2};

use super::{clamp_pan_inertia_center, pan_camera_by_pixels, pan_center_to_anchor};
use crate::{
    coords::LatLon,
    projection::globe::camera::{GlobeCameraOptions, GlobeCameraState},
};

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
}

fn camera(center: LatLon, zoom: f64) -> GlobeCameraState {
    GlobeCameraState::new(GlobeCameraOptions {
        width: 512.0,
        height: 512.0,
        field_of_view_degrees: 36.869_897_645_844_02,
        center,
        world_size: 512.0 * 2_f64.powf(zoom),
        bearing_degrees: 0.0,
        pitch_degrees: 0.0,
        roll_degrees: 0.0,
        center_offset: Point2::new(0.0, 0.0),
    })
    .unwrap_or_else(|error| panic!("camera must be valid: {error}"))
}

#[test]
fn identical_anchor_leaves_center_and_zoom_unchanged() {
    let center = LatLon::new(15.0, 10.0);
    let update = pan_center_to_anchor(center, 20.0, center, center);

    assert_close(update.center.latitude, center.latitude);
    assert_close(update.center.longitude, center.longitude);
    assert_close(update.zoom_adjustment, 0.0);
}

#[test]
fn pan_rotation_stays_finite_near_pole_and_antimeridian() {
    let update = pan_center_to_anchor(
        LatLon::new(80.0, 179.0),
        0.0,
        LatLon::new(85.0, -179.0),
        LatLon::new(84.0, 175.0),
    );

    assert!(update.center.latitude.is_finite());
    assert!(update.center.longitude.is_finite());
    assert!(update.zoom_adjustment.is_finite());
    assert!((-90.0..=90.0).contains(&update.center.latitude));
    assert!((-180.0..180.0).contains(&update.center.longitude));
}

#[test]
fn horizontal_drag_rotates_center_in_opposite_direction() {
    let update = pan_center_to_anchor(
        LatLon::new(0.0, 0.0),
        0.0,
        LatLon::new(0.0, -10.0),
        LatLon::new(0.0, 0.0),
    );

    assert_close(update.center.latitude, 0.0);
    assert_close(update.center.longitude, -10.0);
}

#[test]
fn inertia_target_is_limited_to_intended_antimeridian_direction() {
    let positive = clamp_pan_inertia_center(LatLon::new(10.0, 0.0), LatLon::new(20.0, 200.0));
    let negative = clamp_pan_inertia_center(LatLon::new(10.0, 0.0), LatLon::new(20.0, -200.0));

    assert_close(positive.longitude, 179.5);
    assert_close(negative.longitude, -179.5);
    assert_close(positive.latitude, 20.0);
}

#[test]
fn pixel_pan_preserves_bearing_and_changes_latitude() {
    let camera = camera(LatLon::new(20.0, 0.0), 4.0);
    let update = pan_camera_by_pixels(
        &camera,
        Point2::new(256.0, 256.0),
        Point2::new(256.0, 256.0),
        Vector2::new(50.0, 30.0),
    )
    .unwrap_or_else(|| panic!("finite drag must produce a pan update"));

    assert_ne!(update.center.latitude, 20.0);
    assert!(update.zoom_adjustment.is_finite());
}

#[test]
fn off_sphere_grab_uses_center_anchor() {
    let camera = camera(LatLon::new(0.0, 0.0), 1.0);
    let delta = Vector2::new(50.0, 30.0);
    let center = Point2::new(256.0, 256.0);
    let off_sphere = pan_camera_by_pixels(&camera, Point2::new(500.0, 30.0), center, delta)
        .unwrap_or_else(|| panic!("off-sphere drag must use the center"));
    let centered = pan_camera_by_pixels(&camera, center, center, delta)
        .unwrap_or_else(|| panic!("center drag must produce a pan update"));

    assert_close(off_sphere.center.longitude, centered.center.longitude);
    assert_close(off_sphere.center.latitude, centered.center.latitude);
}

#[test]
fn on_sphere_grab_uses_requested_anchor() {
    let camera = camera(LatLon::new(0.0, 0.0), 1.0);
    let delta = Vector2::new(50.0, 30.0);
    let center = Point2::new(256.0, 256.0);
    let dragged = pan_camera_by_pixels(&camera, Point2::new(300.0, 256.0), center, delta)
        .unwrap_or_else(|| panic!("on-sphere drag must produce a pan update"));
    let centered = pan_camera_by_pixels(&camera, center, center, delta)
        .unwrap_or_else(|| panic!("center drag must produce a pan update"));

    assert_ne!(dragged.center.longitude, centered.center.longitude);
}
