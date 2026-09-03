use cgmath::Point2;

use super::{covering_tiles, GlobeCoveringError, GlobeCoveringOptions};
use crate::{
    coords::{LatLon, ZoomLevel, TILE_SIZE},
    projection::globe::{camera::GlobeCameraOptions, covering::TileElevationRange},
};

fn camera(width: f64, height: f64, center: LatLon, zoom: f64) -> super::GlobeCameraState {
    super::GlobeCameraState::new(GlobeCameraOptions {
        width,
        height,
        field_of_view_degrees: 36.869_897_645_844_02,
        center,
        world_size: TILE_SIZE * 2_f64.powf(zoom),
        bearing_degrees: 0.0,
        pitch_degrees: 0.0,
        roll_degrees: 0.0,
        center_offset: Point2::new(0.0, 0.0),
    })
    .expect("reference camera should be valid")
}

fn options(zoom: u8) -> GlobeCoveringOptions {
    GlobeCoveringOptions {
        zoom: ZoomLevel::new(zoom),
        requested_zoom: f64::from(zoom),
        variable_zoom: false,
        padding: 0,
        max_tiles: 512,
        elevation: TileElevationRange::default(),
    }
}

#[test]
fn zoomed_out_matches_gl_js_reference() {
    let tiles = covering_tiles(
        &camera(128.0, 128.0, LatLon::new(0.0, 0.0), -1.0),
        options(0),
    )
    .expect("covering should succeed");

    assert_eq!(tiles, [(0, 0, ZoomLevel::new(0)).into()]);
}

#[test]
fn zoom_three_matches_gl_js_reference() {
    let tiles = covering_tiles(
        &camera(128.0, 128.0, LatLon::new(0.01, -0.02), 3.0),
        options(3),
    )
    .expect("covering should succeed");
    let expected = [
        (3, 3, ZoomLevel::new(3)).into(),
        (3, 4, ZoomLevel::new(3)).into(),
        (4, 3, ZoomLevel::new(3)).into(),
        (4, 4, ZoomLevel::new(3)).into(),
    ];

    assert_eq!(tiles, expected);
}

#[test]
fn loose_padding_wraps_across_antimeridian_without_world_copies() {
    let mut covering_options = options(3);
    covering_options.padding = 1;
    covering_options.max_tiles = 64;
    let tiles = covering_tiles(
        &camera(64.0, 64.0, LatLon::new(0.0, 179.99), 3.0),
        covering_options,
    )
    .expect("covering should succeed");

    assert!(tiles.iter().all(|tile| (0..8).contains(&tile.x)));
    assert!(tiles.iter().any(|tile| tile.x == 0));
    assert!(tiles.iter().any(|tile| tile.x == 7));
}

#[test]
fn unsupported_zoom_is_rejected_before_traversal() {
    let error = covering_tiles(
        &camera(128.0, 128.0, LatLon::new(0.0, 0.0), 3.0),
        options(32),
    )
    .expect_err("zoom 32 is not representable by world tile coordinates");

    assert_eq!(error, GlobeCoveringError::UnsupportedZoom { zoom: 32 });
}

#[test]
fn pitched_view_matches_gl_js_variable_lod_reference() {
    let globe = super::GlobeCameraState::new(GlobeCameraOptions {
        width: 128.0,
        height: 128.0,
        field_of_view_degrees: 36.869_897_645_844_02,
        center: LatLon::new(0.001, -0.002),
        world_size: TILE_SIZE * 256.0,
        bearing_degrees: 0.0,
        pitch_degrees: 80.0,
        roll_degrees: 0.0,
        center_offset: Point2::new(0.0, 0.0),
    })
    .expect("pitched reference camera should be valid");
    let mut covering_options = options(8);
    covering_options.variable_zoom = true;
    covering_options.elevation.max_meters = super::elevation_for_tile_culling(&globe, 0.0);
    let tiles = covering_tiles(&globe, covering_options).expect("covering should succeed");
    let expected = [
        (32, 31, ZoomLevel::new(6)).into(),
        (31, 31, ZoomLevel::new(6)).into(),
        (511, 512, ZoomLevel::new(10)).into(),
        (512, 512, ZoomLevel::new(10)).into(),
        (511, 513, ZoomLevel::new(10)).into(),
        (512, 513, ZoomLevel::new(10)).into(),
    ];

    assert_eq!(tiles, expected);
}

#[test]
fn pitched_rotated_view_matches_gl_js_variable_lod_reference() {
    let globe = super::GlobeCameraState::new(GlobeCameraOptions {
        width: 128.0,
        height: 128.0,
        field_of_view_degrees: 36.869_897_645_844_02,
        center: LatLon::new(0.001, -0.002),
        world_size: TILE_SIZE * 256.0,
        bearing_degrees: 45.0,
        pitch_degrees: 80.0,
        roll_degrees: 0.0,
        center_offset: Point2::new(0.0, 0.0),
    })
    .expect("rotated reference camera should be valid");
    let mut covering_options = options(8);
    covering_options.variable_zoom = true;
    covering_options.elevation.max_meters = super::elevation_for_tile_culling(&globe, 0.0);
    let tiles = covering_tiles(&globe, covering_options).expect("covering should succeed");
    let expected = [
        (64, 64, ZoomLevel::new(7)).into(),
        (64, 63, ZoomLevel::new(7)).into(),
        (63, 63, ZoomLevel::new(7)).into(),
        (510, 512, ZoomLevel::new(10)).into(),
        (511, 512, ZoomLevel::new(10)).into(),
        (511, 513, ZoomLevel::new(10)).into(),
    ];

    assert_eq!(tiles, expected);
}

#[test]
fn antimeridian_view_selects_both_canonical_edges() {
    let globe = camera(128.0, 128.0, LatLon::new(-0.001, 179.99), 5.0);
    let mut covering_options = options(5);
    covering_options.variable_zoom = true;
    let tiles = covering_tiles(&globe, covering_options).expect("covering should succeed");
    let expected = [
        (31, 16, ZoomLevel::new(5)).into(),
        (31, 15, ZoomLevel::new(5)).into(),
        (0, 16, ZoomLevel::new(5)).into(),
        (0, 15, ZoomLevel::new(5)).into(),
    ];

    assert_eq!(tiles, expected);
}
