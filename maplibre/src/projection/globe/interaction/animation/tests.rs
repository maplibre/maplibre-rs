#![allow(clippy::panic)]

use super::{jump_to_target, GlobeEase, GlobeFly};
use crate::{coords::LatLon, projection::globe::globe_zoom_adjustment};

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
}

#[test]
fn omitted_jump_zoom_preserves_apparent_globe_size() {
    let target = jump_to_target(LatLon::new(0.0, 0.0), 3.0, LatLon::new(60.0, 10.0), None);

    assert_close(target.zoom, 2.0);
}

#[test]
fn ease_uses_short_antimeridian_path_and_reaches_exact_target() {
    let ease = GlobeEase::new(
        LatLon::new(10.0, 170.0),
        4.0,
        LatLon::new(30.0, -170.0),
        Some(5.0),
    );
    let midpoint = ease.sample(0.5);
    let end = ease.sample(1.0);

    assert!(midpoint.center.longitude.abs() > 170.0);
    assert_close(end.center.latitude, 30.0);
    assert_close(end.center.longitude, -170.0);
    assert_close(end.zoom, 5.0);
}

#[test]
fn ease_without_zoom_compensates_latitude_at_every_endpoint() {
    let ease = GlobeEase::new(LatLon::new(0.0, 0.0), 3.0, LatLon::new(60.0, 20.0), None);

    assert_close(ease.target().zoom, 3.0 + globe_zoom_adjustment(0.0, 60.0));
    assert_close(ease.sample(0.0).zoom, 3.0);
    assert_close(ease.sample(1.0).zoom, 2.0);
}

#[test]
fn flight_exports_globe_distance_and_exact_final_state() {
    let flight = GlobeFly::new(
        512.0,
        LatLon::new(0.0, 0.0),
        2.0,
        LatLon::new(0.0, 90.0),
        Some(3.0),
        0.0,
    );

    assert_close(flight.pixel_path_length, 128.0);
    assert_close(flight.scale_of_zoom, 2.0);
    assert_close(flight.scale_of_min_zoom, 0.25);
    let target = flight.target();
    let end = flight.sample(1.0, 0.5, 0.75);
    assert_close(end.center.latitude, target.center.latitude);
    assert_close(end.center.longitude, target.center.longitude);
    assert_close(end.zoom, target.zoom);
}
