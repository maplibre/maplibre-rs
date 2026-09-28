#![allow(clippy::expect_used, clippy::panic)]

use super::{project_tile_coordinates_to_unit_sphere, ProjectionSpecification, ProjectionType};
use crate::{coords::EXTENT, style::Style};

fn assert_close(left: f64, right: f64) {
    assert!((left - right).abs() < 1e-12, "{left} != {right}");
}

#[test]
fn parses_globe_projection_specification() {
    let projection: ProjectionSpecification =
        serde_json::from_str(r#"{"type":"globe"}"#).expect("projection should deserialize");

    assert_eq!(projection.projection_type, ProjectionType::Globe);
}

#[test]
fn parses_vertical_perspective_projection_specification() {
    let specification: ProjectionSpecification =
        serde_json::from_str(r#"{"type":"vertical-perspective"}"#)
            .expect("vertical-perspective projection should deserialize");

    assert_eq!(
        specification.projection_type,
        ProjectionType::VerticalPerspective
    );
    assert_eq!(specification.projection_type.globe_transition(30.0), 1.0);
}

#[test]
fn projection_expression_interpolates_and_round_trips() {
    let json =
        r#"{"type":["interpolate",["linear"],["zoom"],10,"vertical-perspective",12,"mercator"]}"#;
    let specification: ProjectionSpecification =
        serde_json::from_str(json).expect("projection expression should deserialize");

    assert_eq!(specification.projection_type.globe_transition(9.0), 1.0);
    assert_eq!(specification.projection_type.globe_transition(11.0), 0.5);
    assert_eq!(specification.projection_type.globe_transition(13.0), 0.0);
    assert_eq!(
        serde_json::from_str::<ProjectionSpecification>(
            &serde_json::to_string(&specification).expect("projection should serialize"),
        )
        .expect("serialized projection should deserialize"),
        specification
    );
}

#[test]
fn projection_expression_rejects_invalid_zoom_stops() {
    let result = serde_json::from_str::<ProjectionSpecification>(
        r#"{"type":["interpolate",["linear"],["zoom"],12,"vertical-perspective",10,"mercator"]}"#,
    );

    assert!(result.is_err());
}

#[test]
fn defaults_to_mercator_projection() {
    assert_eq!(ProjectionType::default(), ProjectionType::Mercator);
}

#[test]
fn parses_globe_projection_from_style() {
    let style: Style = serde_json::from_str(
        r#"{"version":8,"sources":{},"layers":[],"projection":{"type":"globe"}}"#,
    )
    .expect("style should deserialize");

    assert_eq!(
        style.projection.map(|value| value.projection_type),
        Some(ProjectionType::Globe)
    );
}

#[test]
fn style_projection_is_optional() {
    let style: Style = serde_json::from_str(r#"{"version":8,"sources":{},"layers":[]}"#)
        .expect("style should deserialize");

    assert_eq!(style.projection, None);
}

#[test]
fn projects_prime_meridian_and_equator_to_positive_z() {
    let point = project_tile_coordinates_to_unit_sphere(0, 0, 0, EXTENT / 2.0, EXTENT / 2.0);

    assert_close(point.x, 0.0);
    assert_close(point.y, 0.0);
    assert_close(point.z, 1.0);
}

#[test]
fn projected_points_stay_on_unit_sphere() {
    let point = project_tile_coordinates_to_unit_sphere(2, 1, 3, 1234.0, 3456.0);

    assert_close(
        point.x * point.x + point.y * point.y + point.z * point.z,
        1.0,
    );
}

#[test]
fn adjacent_tiles_share_the_same_projected_edge() {
    let left_edge = project_tile_coordinates_to_unit_sphere(0, 0, 1, EXTENT, EXTENT / 2.0);
    let right_edge = project_tile_coordinates_to_unit_sphere(1, 0, 1, 0.0, EXTENT / 2.0);

    assert_close(left_edge.x, right_edge.x);
    assert_close(left_edge.y, right_edge.y);
    assert_close(left_edge.z, right_edge.z);
}

#[test]
fn web_mercator_bounds_map_to_opposite_latitudes() {
    let north = project_tile_coordinates_to_unit_sphere(0, 0, 0, EXTENT / 2.0, 0.0);
    let south = project_tile_coordinates_to_unit_sphere(0, 0, 0, EXTENT / 2.0, EXTENT);

    assert_close(north.y, -south.y);
    assert!(north.y > 0.99);
}
