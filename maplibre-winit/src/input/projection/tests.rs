#![allow(clippy::expect_used, clippy::panic)]

use cgmath::{Deg, Vector2};
use maplibre::{
    coords::{LatLon, WorldCoords, Zoom},
    projection::{ProjectionSpecification, ProjectionType},
    render::view_state::ViewState,
    style::Style,
    window::PhysicalSize,
};

use super::{active_globe_camera, zoom_globe_around_pixel};

fn globe_view(zoom: f64) -> (Style, ViewState) {
    let zoom = Zoom::new(zoom);
    let style = Style {
        projection: Some(ProjectionSpecification {
            projection_type: ProjectionType::Globe,
        }),
        ..Default::default()
    };
    let view = ViewState::new(
        PhysicalSize::new(800, 600).expect("valid viewport"),
        WorldCoords::from_lat_lon(LatLon::new(35.0, 120.0), zoom),
        zoom,
        Deg(0.0),
        Deg(45.0),
    );
    (style, view)
}

#[test]
fn center_zoom_preserves_geographic_center() {
    let (style, mut view) = globe_view(2.0);
    let before = active_globe_camera(&style, &view).expect("globe camera");
    assert!(zoom_globe_around_pixel(
        &style,
        &mut view,
        Vector2::new(400.0, 300.0),
        Zoom::new(3.0),
    ));
    let after = active_globe_camera(&style, &view).expect("zoomed globe camera");
    assert!((before.center().latitude - after.center().latitude).abs() < 1e-8);
    assert!((before.center().longitude - after.center().longitude).abs() < 1e-8);
    assert!((view.zoom().value() - 3.0).abs() < 1e-8);
}

#[test]
fn off_globe_zoom_uses_view_center() {
    let (style, mut view) = globe_view(0.0);
    let mut centered = view.clone();
    let pointer = Vector2::new(0.0, 0.0);
    let camera = active_globe_camera(&style, &view).expect("globe camera");
    assert!(!camera.is_point_on_map_surface(cgmath::Point2::new(pointer.x, pointer.y)));
    assert!(zoom_globe_around_pixel(
        &style,
        &mut view,
        pointer,
        Zoom::new(0.5)
    ));
    assert!(zoom_globe_around_pixel(
        &style,
        &mut centered,
        Vector2::new(400.0, 300.0),
        Zoom::new(0.5),
    ));
    let actual = active_globe_camera(&style, &view).expect("zoomed globe camera");
    let expected = active_globe_camera(&style, &centered).expect("centered globe camera");
    assert!((actual.center().latitude - expected.center().latitude).abs() < 1e-8);
    assert!((actual.center().longitude - expected.center().longitude).abs() < 1e-8);
    assert!((view.zoom().value() - centered.zoom().value()).abs() < 1e-8);
}

#[test]
fn zoom_into_mercator_preserves_geographic_center() {
    let (style, mut view) = globe_view(11.5);
    assert!(zoom_globe_around_pixel(
        &style,
        &mut view,
        Vector2::new(400.0, 300.0),
        Zoom::new(12.5),
    ));
    let expected = WorldCoords::from_lat_lon(LatLon::new(35.0, 120.0), view.zoom());
    let actual = view.camera().position();
    assert!((actual.x - expected.x).abs() < 1e-6);
    assert!((actual.y - expected.y).abs() < 1e-6);
    assert!(active_globe_camera(&style, &view).is_none());
}
