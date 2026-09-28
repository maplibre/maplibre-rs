#![allow(clippy::panic)]

use cgmath::Vector3;

use super::{zoom_around_globe, GlobeZoomInput};
use crate::coords::LatLon;

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
}

#[test]
fn zero_zoom_delta_preserves_center() {
    let update = zoom_around_globe(GlobeZoomInput {
        start_center: LatLon::new(10.0, 20.0),
        zoom_after_delta: 3.0,
        zoom_delta: 0.0,
        pointer_location: LatLon::new(20.0, 30.0),
        exact_center: LatLon::new(12.0, 24.0),
        ray_origin: Vector3::new(0.0, 0.0, 3.0),
        ray_direction: Vector3::new(0.0, 0.0, -1.0),
        relative_globe_radius: 1.0,
    });

    assert_close(update.center.latitude, 10.0);
    assert_close(update.center.longitude, 20.0);
    assert_close(update.zoom, 3.0);
}

#[test]
fn central_ray_uses_exact_pointer_anchor() {
    let exact = LatLon::new(5.0, 7.0);
    let update = zoom_around_globe(GlobeZoomInput {
        start_center: LatLon::new(0.0, 0.0),
        zoom_after_delta: 4.0,
        zoom_delta: 1.0,
        pointer_location: LatLon::new(10.0, 10.0),
        exact_center: exact,
        ray_origin: Vector3::new(0.0, 0.0, 3.0),
        ray_direction: Vector3::new(0.0, 0.0, -1.0),
        relative_globe_radius: 1.0,
    });

    assert_close(update.center.latitude, exact.latitude);
    assert_close(update.center.longitude, exact.longitude);
}

#[test]
fn grazing_ray_blends_toward_bounded_heuristic() {
    let update = zoom_around_globe(GlobeZoomInput {
        start_center: LatLon::new(0.0, 0.0),
        zoom_after_delta: 2.0,
        zoom_delta: 1.0,
        pointer_location: LatLon::new(80.0, 120.0),
        exact_center: LatLon::new(70.0, 160.0),
        ray_origin: Vector3::new(0.0, 0.0, 3.0),
        ray_direction: Vector3::new(1.0, 0.0, 0.0),
        relative_globe_radius: 0.4,
    });

    assert!(update.center.latitude.abs() <= 85.051_128_779_806_59);
    assert!(update.center.longitude.abs() <= 180.0);
    assert!(update.zoom.is_finite());
}
