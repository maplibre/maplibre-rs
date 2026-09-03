use cgmath::InnerSpace;

use super::initial_view_state;
use crate::{
    coords::{LatLon, Zoom},
    style::{light::LightSpecification, Style},
    window::PhysicalSize,
};

#[test]
fn initial_view_uses_style_camera_options() {
    let style: Style = serde_json::from_str(
        r#"{
            "version": 8,
            "center": [160.0, 20.0],
            "zoom": 3.5,
            "bearing": 45.0,
            "pitch": 30.0,
            "sources": {},
            "layers": []
        }"#,
    )
    .expect("style should parse");
    let view = initial_view_state(
        PhysicalSize::new(512, 512).expect("size should be nonzero"),
        &style,
    );

    assert_eq!(view.zoom().value(), Zoom::new(3.5).value());
    assert!((view.camera().get_roll().0.to_degrees() - 45.0).abs() <= 1e-12);
    assert!((view.camera().get_pitch().0.to_degrees() - 30.0).abs() <= 1e-12);

    let camera_center = crate::render::projection::globe_camera_for_view(&view)
        .expect("globe camera should be valid")
        .center();
    assert!((camera_center.latitude - LatLon::new(20.0, 160.0).latitude).abs() <= 1e-9);
    assert!((camera_center.longitude - LatLon::new(20.0, 160.0).longitude).abs() <= 1e-9);
}

#[test]
fn unrotated_headless_view_keeps_map_light_in_view_axes() {
    let style: Style = serde_json::from_str(
        r#"{
            "version": 8,
            "center": [0.0, 0.0],
            "zoom": 10,
            "sources": {},
            "layers": []
        }"#,
    )
    .expect("style should parse");
    let view = initial_view_state(
        PhysicalSize::new(512, 512).expect("size should be nonzero"),
        &style,
    );
    let camera = crate::render::projection::globe_camera_for_view(&view)
        .expect("globe camera should be valid");
    let map: LightSpecification =
        serde_json::from_str(r#"{"anchor":"map","position":[1.5,0,180]}"#)
            .expect("map light should parse");
    let viewport: LightSpecification =
        serde_json::from_str(r#"{"anchor":"viewport","position":[1.5,0,180]}"#)
            .expect("viewport light should parse");

    let map_direction = map
        .sun_direction_in_view(&camera, 10.0)
        .expect("map direction should be valid");
    let viewport_direction = viewport
        .sun_direction_in_view(&camera, 10.0)
        .expect("viewport direction should be valid");
    assert!((map_direction - viewport_direction).magnitude() <= 1e-12);
}
