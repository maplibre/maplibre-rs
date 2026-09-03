#![allow(clippy::expect_used, clippy::panic)]

use cgmath::{Point2, Vector3, Vector4};

use super::{
    allows_variable_zoom, allows_world_copies, distance_to_tile_2d, globe_tile_bounding_volume,
    nearest_tile_wrap, GlobeTileBoundsError, TileElevationRange,
};
use crate::coords::{TileCoords, ZoomLevel};

fn tile(x: u32, y: u32, zoom: u8) -> TileCoords {
    TileCoords {
        x,
        y,
        z: ZoomLevel::new(zoom),
    }
}

fn assert_close(left: f64, right: f64) {
    assert!((left - right).abs() < 1e-10, "{left} != {right}");
}

fn assert_vector_close(left: Vector3<f64>, right: Vector3<f64>) {
    assert_close(left.x, right.x);
    assert_close(left.y, right.y);
    assert_close(left.z, right.z);
}

fn assert_plane_close(left: Vector4<f64>, right: Vector4<f64>) {
    assert_close(left.x, right.x);
    assert_close(left.y, right.y);
    assert_close(left.z, right.z);
    assert_close(left.w, right.w);
}

#[test]
fn zoom_zero_volume_covers_unit_sphere() {
    let volume = globe_tile_bounding_volume(tile(0, 0, 0), TileElevationRange::default())
        .expect("zoom-zero bounds are valid");

    assert_eq!(volume.min, Vector3::new(-1.0, -1.0, -1.0));
    assert_eq!(volume.max, Vector3::new(1.0, 1.0, 1.0));
    assert_eq!(volume.points.len(), 8);
}

#[test]
fn zoom_one_volumes_match_gl_js_quadrants() {
    let west = globe_tile_bounding_volume(tile(0, 0, 1), TileElevationRange::default())
        .expect("western quadrant bounds are valid");
    let east = globe_tile_bounding_volume(tile(1, 0, 1), TileElevationRange::default())
        .expect("eastern quadrant bounds are valid");

    assert_eq!(west.min, Vector3::new(-1.0, 0.0, -1.0));
    assert_eq!(west.max, Vector3::new(0.0, 1.0, 1.0));
    assert_eq!(east.min, Vector3::new(0.0, 0.0, -1.0));
    assert_eq!(east.max, Vector3::new(1.0, 1.0, 1.0));
}

#[test]
fn curved_volume_matches_gl_js_reference_fixture() {
    let volume = globe_tile_bounding_volume(tile(1, 1, 5), TileElevationRange::default())
        .expect("curved tile bounds are valid");

    assert_vector_close(
        volume.min,
        Vector3::new(
            -0.04878262717137475,
            0.9918417649235776,
            -0.1250257487589308,
        ),
    );
    assert_vector_close(
        volume.max,
        Vector3::new(
            -0.020462724105427713,
            0.9944839919477184,
            -0.09690430455523656,
        ),
    );
    assert_eq!(volume.points.len(), 8);
    assert_vector_close(
        volume.points[0],
        Vector3::new(
            -0.040144275638466294,
            0.9946001124628003,
            -0.09691685469802916,
        ),
    );
    assert_plane_close(
        volume.planes[0],
        Vector4::new(
            0.033568258567807485,
            -0.9932912960221243,
            0.11065971834147033,
            1.0,
        ),
    );
    assert_plane_close(
        volume.planes[4],
        Vector4::new(
            0.9238795325112867,
            -3.8143839245115144e-17,
            -0.38268343236509017,
            0.0,
        ),
    );
}

#[test]
fn elevation_expands_radial_bounds() {
    let volume = globe_tile_bounding_volume(
        tile(0, 0, 0),
        TileElevationRange {
            min_meters: -100.0,
            max_meters: 1_000.0,
        },
    )
    .expect("finite elevation range is valid");

    assert!(volume.max.x > 1.0);
    assert!(volume.min.x < -1.0);
}

#[test]
fn invalid_elevation_is_rejected() {
    let error = globe_tile_bounding_volume(
        tile(0, 0, 0),
        TileElevationRange {
            min_meters: f64::NAN,
            max_meters: 0.0,
        },
    )
    .expect_err("non-finite elevation is invalid");

    assert!(matches!(
        error,
        GlobeTileBoundsError::InvalidElevation { .. }
    ));
}

#[test]
fn tile_distance_wraps_antimeridian() {
    let west_tile = tile(0, 1, 2);
    assert_close(distance_to_tile_2d(Point2::new(0.99, 0.3), west_tile), 0.01);
    assert_close(distance_to_tile_2d(Point2::new(0.1, 0.3), west_tile), 0.0);
}

#[test]
fn tile_distance_mirrors_across_poles() {
    let northern = tile(0, 0, 2);
    assert_close(distance_to_tile_2d(Point2::new(0.6, -0.1), northern), 0.0);
}

#[test]
fn nearest_wrap_tracks_center_across_antimeridian() {
    let western = tile(0, 0, 2);
    let eastern = tile(3, 0, 2);
    assert_eq!(nearest_tile_wrap(0.99, western), 1);
    assert_eq!(nearest_tile_wrap(0.01, eastern), -1);
    assert_eq!(nearest_tile_wrap(0.1, western), 0);
}

#[test]
fn globe_covering_policy_matches_reference() {
    assert!(!allows_variable_zoom(4));
    assert!(allows_variable_zoom(5));
    assert!(!allows_world_copies());
}
