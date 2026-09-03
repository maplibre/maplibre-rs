#![allow(clippy::expect_used, clippy::panic)]

use cgmath::{InnerSpace, Point2, SquareMatrix, Vector3};

use super::{GlobeCameraError, GlobeCameraOptions, GlobeCameraState};
use crate::coords::{LatLon, TileCoords, ZoomLevel, EXTENT};

fn options() -> GlobeCameraOptions {
    GlobeCameraOptions {
        width: 512.0,
        height: 512.0,
        field_of_view_degrees: 45.0,
        center: LatLon::new(0.0, 0.0),
        world_size: 512.0,
        bearing_degrees: 0.0,
        pitch_degrees: 0.0,
        roll_degrees: 0.0,
        center_offset: Point2::new(0.0, 0.0),
    }
}

fn assert_close(left: f64, right: f64) {
    assert!((left - right).abs() < 1e-9, "{left} != {right}");
}

fn assert_point_close(left: Point2<f64>, right: Point2<f64>) {
    assert_close(left.x, right.x);
    assert_close(left.y, right.y);
}

#[test]
fn centered_location_projects_to_viewport_center() {
    let camera = GlobeCameraState::new(options()).expect("default globe camera is valid");
    let screen = camera.location_to_screen(LatLon::new(0.0, 0.0), 0.0);

    assert_point_close(screen, Point2::new(256.0, 256.0));
    assert!(!camera.is_location_occluded(LatLon::new(0.0, 0.0)));
    assert!(camera.is_location_occluded(LatLon::new(0.0, 180.0)));
}

#[test]
fn center_screen_ray_round_trips_map_center() {
    let camera = GlobeCameraState::new(options()).expect("default globe camera is valid");
    let location = camera
        .screen_point_to_location(Point2::new(256.0, 256.0))
        .expect("center ray intersects globe");

    assert_close(location.latitude, 0.0);
    assert_close(location.longitude, 0.0);
    assert!(camera.is_point_on_map_surface(Point2::new(256.0, 256.0)));
}

#[test]
fn ray_directions_are_normalized() {
    let camera = GlobeCameraState::new(options()).expect("default globe camera is valid");
    for pixel in [Point2::new(0.0, 0.0), Point2::new(256.0, 256.0)] {
        let ray = camera
            .ray_direction_from_pixel(pixel)
            .expect("finite viewport pixel produces a ray");
        assert_close(ray.magnitude(), 1.0);
    }
}

#[test]
fn screen_misses_clamp_to_horizon() {
    let mut camera_options = options();
    camera_options.world_size = 128.0;
    let camera = GlobeCameraState::new(camera_options).expect("zoomed-out globe camera is valid");

    assert!(!camera.is_point_on_map_surface(Point2::new(0.0, 0.0)));
    assert!(camera
        .screen_point_to_location(Point2::new(0.0, 0.0))
        .is_some());
}

#[test]
fn tile_projection_matches_geographic_projection() {
    let camera = GlobeCameraState::new(options()).expect("default globe camera is valid");
    let tile_projection = camera.project_tile_coordinates(
        EXTENT / 2.0,
        EXTENT / 2.0,
        TileCoords {
            x: 0,
            y: 0,
            z: ZoomLevel::new(0),
        },
        0.0,
    );
    let screen = camera.location_to_screen(LatLon::new(0.0, 0.0), 0.0);

    assert_close((tile_projection.point.x * 0.5 + 0.5) * 512.0, screen.x);
    assert_close((-tile_projection.point.y * 0.5 + 0.5) * 512.0, screen.y);
    assert!(!tile_projection.is_occluded);
}

#[test]
fn scale_and_text_corrections_match_reference_values() {
    let mut camera_options = options();
    camera_options.center = LatLon::new(60.0, 0.0);
    let camera =
        GlobeCameraState::new(camera_options).expect("high-latitude globe camera is valid");

    assert_close(camera.pixel_scale(), 2.0);
    assert_close(camera.circle_radius_correction(), 0.5);
    assert_close(
        camera.pitched_text_correction(
            EXTENT / 2.0,
            EXTENT / 2.0,
            TileCoords {
                x: 0,
                y: 0,
                z: ZoomLevel::new(0),
            },
        ),
        0.5,
    );
}

#[test]
fn light_direction_uses_globe_tangent_axes() {
    let camera = GlobeCameraState::new(options()).expect("default globe camera is valid");

    assert_eq!(
        camera.transform_light_direction(Vector3::unit_x()),
        Some(Vector3::unit_x())
    );
    assert_eq!(
        camera.transform_light_direction(Vector3::unit_y()),
        Some(-Vector3::unit_y())
    );
    assert_eq!(
        camera.transform_light_direction(Vector3::unit_z()),
        Some(Vector3::unit_z())
    );
    assert_eq!(
        camera.transform_light_direction(Vector3::new(0.0, 0.0, 0.0)),
        None
    );
}

#[test]
fn matrices_are_invertible_and_depth_contains_globe() {
    let camera = GlobeCameraState::new(options()).expect("default globe camera is valid");
    let identity = camera.view_projection() * camera.inverse_view_projection();
    assert!(identity.is_identity());

    let (near, far) = camera.depth_range();
    assert_eq!(near, 0.5);
    assert!(far > near + camera.globe_radius_pixels() * 2.0);
}

#[test]
fn invalid_options_return_typed_errors() {
    let invalid_viewport = GlobeCameraState::new(GlobeCameraOptions {
        width: 0.0,
        ..options()
    })
    .expect_err("zero-width viewport is invalid");
    assert!(matches!(
        invalid_viewport,
        GlobeCameraError::InvalidViewport { .. }
    ));

    let invalid_fov = GlobeCameraState::new(GlobeCameraOptions {
        field_of_view_degrees: 180.0,
        ..options()
    })
    .expect_err("180 degree field of view is invalid");
    assert!(matches!(
        invalid_fov,
        GlobeCameraError::InvalidFieldOfView { .. }
    ));
}
