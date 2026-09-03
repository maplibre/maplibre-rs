#![allow(clippy::expect_used, clippy::panic)]

use cgmath::{InnerSpace, Matrix4, SquareMatrix, Vector3, Vector4};

use super::{
    compose_projection_data, compute_globe_clipping_plane, tile_mercator_coordinates,
    GlobeViewGeometry, ProjectionDataError, ProjectionDataParams, ProjectionMatrices,
};
use crate::coords::{LatLon, TileCoords, ZoomLevel, EXTENT};

fn assert_close(left: f64, right: f64) {
    assert!((left - right).abs() < 1e-10, "{left} != {right}");
}

fn test_matrices() -> ProjectionMatrices {
    ProjectionMatrices {
        mercator: Matrix4::identity(),
        globe: Matrix4::from_scale(2.0),
    }
}

fn equatorial_view() -> GlobeViewGeometry {
    GlobeViewGeometry {
        center: LatLon::new(0.0, 0.0),
        bearing_degrees: 0.0,
        pitch_degrees: 0.0,
        camera_to_center_distance: 1.0,
        globe_radius_pixels: 1.0,
    }
}

#[test]
fn tile_coordinates_match_gl_js_fixture() {
    let coordinates = tile_mercator_coordinates(Some(TileCoords {
        x: 1,
        y: 0,
        z: ZoomLevel::new(1),
    }));

    assert_eq!(
        coordinates,
        Vector4::new(0.5, 0.0, (0.5 / EXTENT) as f32, (0.5 / EXTENT) as f32)
    );
}

#[test]
fn world_space_coordinates_cover_zoom_zero_tile() {
    let coordinates = tile_mercator_coordinates(None);
    assert_eq!(
        coordinates,
        Vector4::new(0.0, 0.0, (1.0 / EXTENT) as f32, (1.0 / EXTENT) as f32)
    );
}

#[test]
fn transition_selects_globe_matrix_but_respects_draw_opt_out() {
    let data = compose_projection_data(
        test_matrices(),
        Vector4::unit_w(),
        0.5,
        ProjectionDataParams::default(),
    );

    assert_eq!(data.main_matrix, Matrix4::from_scale(2.0));
    assert_eq!(data.projection_transition, 0.0);
    assert_eq!(data.fallback_matrix, Matrix4::identity());
}

#[test]
fn transition_is_clamped_and_applied_per_draw() {
    let data = compose_projection_data(
        test_matrices(),
        Vector4::unit_w(),
        2.0,
        ProjectionDataParams {
            apply_globe_matrix: true,
            ..ProjectionDataParams::default()
        },
    );

    assert_eq!(data.projection_transition, 1.0);
}

#[test]
fn non_finite_transition_falls_back_to_mercator() {
    let data = compose_projection_data(
        test_matrices(),
        Vector4::unit_w(),
        f32::NAN,
        ProjectionDataParams {
            apply_globe_matrix: true,
            ..ProjectionDataParams::default()
        },
    );

    assert_eq!(data.main_matrix, Matrix4::identity());
    assert_eq!(data.projection_transition, 0.0);
}

#[test]
fn only_zoom_zero_tile_enables_antimeridian_fragment_clipping() {
    for (zoom, expected) in [(0, true), (1, false)] {
        let data = compose_projection_data(
            test_matrices(),
            Vector4::unit_w(),
            1.0,
            ProjectionDataParams {
                tile: Some(TileCoords {
                    x: 0,
                    y: 0,
                    z: ZoomLevel::new(zoom),
                }),
                ..ProjectionDataParams::default()
            },
        );
        assert_eq!(data.clip_antimeridian, expected);
    }
}

#[test]
fn equatorial_clipping_plane_matches_reference_geometry() {
    let plane = compute_globe_clipping_plane(equatorial_view())
        .expect("positive view dimensions produce a clipping plane");

    assert_close(plane.x, 0.0);
    assert_close(plane.y, 0.0);
    assert_close(plane.z, 1.0);
    assert_close(plane.w, -0.5);
    assert!(plane.dot(Vector4::new(0.0, 0.0, 2.0, 1.0)) > 0.0);
    assert!(plane.dot(Vector4::new(0.0, 0.0, 1.0, 1.0)) > 0.0);
    assert!(plane.dot(Vector4::new(1.0, 0.0, 0.0, 1.0)) < 0.0);
}

#[test]
fn clipping_plane_normal_stays_normalized_after_map_rotations() {
    let plane = compute_globe_clipping_plane(GlobeViewGeometry {
        center: LatLon::new(47.0, -122.0),
        bearing_degrees: 31.0,
        pitch_degrees: 50.0,
        ..equatorial_view()
    })
    .expect("rotated view dimensions are valid");

    assert_close(Vector3::new(plane.x, plane.y, plane.z).magnitude(), 1.0);
    assert!(plane.w < 0.0);
}

#[test]
fn invalid_view_geometry_is_rejected() {
    let invalid_radius = compute_globe_clipping_plane(GlobeViewGeometry {
        globe_radius_pixels: 0.0,
        ..equatorial_view()
    })
    .expect_err("zero radius is invalid");
    assert_eq!(
        invalid_radius,
        ProjectionDataError::InvalidGlobeRadius { radius: 0.0 }
    );

    let invalid_distance = compute_globe_clipping_plane(GlobeViewGeometry {
        camera_to_center_distance: -1.0,
        ..equatorial_view()
    })
    .expect_err("negative camera distance is invalid");
    assert_eq!(
        invalid_distance,
        ProjectionDataError::InvalidCameraDistance { distance: -1.0 }
    );

    let invalid_pitch = compute_globe_clipping_plane(GlobeViewGeometry {
        pitch_degrees: f64::NAN,
        ..equatorial_view()
    })
    .expect_err("non-finite pitch is invalid");
    assert!(matches!(
        invalid_pitch,
        ProjectionDataError::InvalidAngle { name: "pitch", .. }
    ));
}
