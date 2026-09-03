use cgmath::{InnerSpace, Point2};

use super::{LightAnchor, LightError, LightSpecification};
use crate::{
    coords::LatLon,
    projection::globe::camera::{GlobeCameraOptions, GlobeCameraState},
    style::Style,
};

fn camera(center: LatLon) -> GlobeCameraState {
    GlobeCameraState::new(GlobeCameraOptions {
        width: 512.0,
        height: 512.0,
        field_of_view_degrees: 45.0,
        center,
        world_size: 1024.0,
        bearing_degrees: 0.0,
        pitch_degrees: 0.0,
        roll_degrees: 0.0,
        center_offset: Point2::new(0.0, 0.0),
    })
    .expect("camera should be valid")
}

#[test]
fn defaults_match_the_gl_js_light_contract() {
    let light: LightSpecification = serde_json::from_str("{}").expect("light should parse");
    assert_eq!(light.anchor, LightAnchor::Viewport);

    let direction = light
        .sun_direction_in_view(&camera(LatLon::new(0.0, 0.0)), 0.0)
        .expect("default direction should be valid");
    assert_close(direction.magnitude(), 1.0);
}

#[test]
fn root_style_parses_gl_js_atmosphere_fixture_light() {
    let style: Style = serde_json::from_str(
        r#"{
            "version": 8,
            "sources": {},
            "layers": [],
            "projection": {"type": "globe"},
            "light": {"anchor": "map", "position": [1.5, 90, 90]},
            "sky": {"atmosphere-blend": 1.0}
        }"#,
    )
    .expect("GL JS atmosphere style should parse");

    assert_eq!(
        style.light.expect("light should exist").anchor,
        LightAnchor::Map
    );
}

#[test]
fn viewport_light_uses_gl_js_spherical_axis_convention() {
    let light: LightSpecification =
        serde_json::from_str(r#"{"anchor":"viewport","position":[1.5,90,90]}"#)
            .expect("light should parse");
    let direction = light
        .sun_direction_in_view(&camera(LatLon::new(0.0, 0.0)), 1.0)
        .expect("direction should be valid");

    assert_close(direction.x, 1.0);
    assert_close(direction.y, 0.0);
    assert_close(direction.z, 0.0);
}

#[test]
fn map_light_rotates_with_geographic_center() {
    let light: LightSpecification =
        serde_json::from_str(r#"{"anchor":"map","position":[1.5,90,90]}"#)
            .expect("light should parse");
    let prime = light
        .sun_direction_in_view(&camera(LatLon::new(0.0, 0.0)), 1.0)
        .expect("direction should be valid");
    let rotated = light
        .sun_direction_in_view(&camera(LatLon::new(0.0, 160.0)), 1.0)
        .expect("direction should be valid");

    assert_close(prime.magnitude(), 1.0);
    assert_close(rotated.magnitude(), 1.0);
    assert!(prime.dot(rotated) < 0.0);
}

#[test]
fn map_light_matches_gl_js_rotation_order() {
    let light: LightSpecification =
        serde_json::from_str(r#"{"anchor":"map","position":[1.5,90,90]}"#)
            .expect("light should parse");
    let direction = light
        .sun_direction_in_view(&camera(LatLon::new(0.0, 160.0)), 1.0)
        .expect("direction should be valid");

    assert_close(direction.x, -160.0_f64.to_radians().cos().abs());
    assert_close(direction.y, 0.0);
    assert_close(direction.z, 160.0_f64.to_radians().sin().abs());
}

#[test]
fn map_and_viewport_lights_match_at_unrotated_origin() {
    let map: LightSpecification =
        serde_json::from_str(r#"{"anchor":"map","position":[1.5,0,180]}"#)
            .expect("map light should parse");
    let viewport: LightSpecification =
        serde_json::from_str(r#"{"anchor":"viewport","position":[1.5,0,180]}"#)
            .expect("viewport light should parse");
    let camera = camera(LatLon::new(0.0, 0.0));
    let map_direction = map
        .sun_direction_in_view(&camera, 10.0)
        .expect("map direction should be valid");
    let viewport_direction = viewport
        .sun_direction_in_view(&camera, 10.0)
        .expect("viewport direction should be valid");

    assert_close(map_direction.x, viewport_direction.x);
    assert_close(map_direction.y, viewport_direction.y);
    assert_close(map_direction.z, viewport_direction.z);
}

#[test]
fn legacy_zoom_stops_interpolate_position() {
    let light: LightSpecification =
        serde_json::from_str(r#"{"position":{"stops":[[0,[1,0,90]],[10,[1,180,90]]]}}"#)
            .expect("light should parse");
    let direction = light
        .sun_direction_in_view(&camera(LatLon::new(0.0, 0.0)), 5.0)
        .expect("direction should be valid");

    assert_close(direction.x, 1.0);
    assert_close(direction.y, 0.0);
}

#[test]
fn zero_radius_is_rejected() {
    let light: LightSpecification =
        serde_json::from_str(r#"{"position":[0,0,0]}"#).expect("light should parse");
    assert_eq!(
        light.sun_direction_in_view(&camera(LatLon::new(0.0, 0.0)), 0.0),
        Err(LightError::InvalidPosition)
    );
}

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() <= 1e-12, "{actual} != {expected}");
}
