#![allow(clippy::expect_used, clippy::panic)]

use std::time::Duration;

use cgmath::Vector2;
use instant::Instant;

use super::{ease, ease_out, PanInertia};

#[test]
fn easing_curve_is_anchored_and_monotonic() {
    assert!((ease(0.0)).abs() < 1e-9);
    assert!((ease(1.0) - 1.0).abs() < 1e-6);
    let mut previous = 0.0;
    for step in 1..=20 {
        let value = ease(f64::from(step) / 20.0);
        assert!(value >= previous, "easing must not reverse at step {step}");
        previous = value;
    }
    assert!(
        ease(0.5) > 0.5,
        "the curve eases out, so it is front-loaded"
    );
}

#[test]
fn release_speed_follows_gl_js_calculate_easing() {
    // 100 px in 100 ms: speed 300 px/s, duration 0.4 s, distance 60 px.
    let easing = ease_out(100.0, 0.1).expect("moving drag eases out");

    assert!((easing.duration.as_secs_f64() - 0.4).abs() < 1e-9);
    assert!((easing.distance - 60.0).abs() < 1e-9);
}

#[test]
fn release_speed_is_capped() {
    let easing = ease_out(10_000.0, 0.1).expect("fast drag eases out");

    assert!((easing.duration.as_secs_f64() - 1400.0 / 750.0).abs() < 1e-9);
}

#[test]
fn a_single_sample_does_not_start_motion() {
    let start = Instant::now();
    let mut inertia = PanInertia::default();
    inertia.record(start, Vector2::new(10.0, 0.0));

    assert!(!inertia.release(start));
    assert!(inertia.step(start).is_none());
}

#[test]
fn stale_samples_are_ignored() {
    let start = Instant::now();
    let mut inertia = PanInertia::default();
    inertia.record(start, Vector2::new(50.0, 0.0));
    inertia.record(start + Duration::from_millis(50), Vector2::new(50.0, 0.0));

    assert!(!inertia.release(start + Duration::from_millis(500)));
}

#[test]
fn motion_delivers_the_eased_distance_in_the_drag_direction() {
    let start = Instant::now();
    let mut inertia = PanInertia::default();
    inertia.record(start, Vector2::new(0.0, 0.0));
    inertia.record(start + Duration::from_millis(100), Vector2::new(60.0, 80.0));
    let released = start + Duration::from_millis(100);

    assert!(inertia.release(released));
    let mut total = Vector2::new(0.0, 0.0);
    for frame in 1..=40 {
        if let Some(delta) = inertia.step(released + Duration::from_millis(frame * 10)) {
            total += delta;
        }
    }
    // 100 px in 100 ms eases out over 60 px along the same direction.
    assert!((total.x - 36.0).abs() < 1e-6, "{total:?}");
    assert!((total.y - 48.0).abs() < 1e-6, "{total:?}");
    assert!(inertia.step(released + Duration::from_secs(1)).is_none());
}

#[test]
fn recording_cancels_a_running_ease_out() {
    let start = Instant::now();
    let mut inertia = PanInertia::default();
    inertia.record(start, Vector2::new(0.0, 0.0));
    inertia.record(start + Duration::from_millis(100), Vector2::new(100.0, 0.0));
    assert!(inertia.release(start + Duration::from_millis(100)));

    inertia.record(start + Duration::from_millis(120), Vector2::new(1.0, 0.0));

    assert!(inertia.step(start + Duration::from_millis(130)).is_none());
}
