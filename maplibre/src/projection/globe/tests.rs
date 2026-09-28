#![allow(clippy::expect_used, clippy::panic)]

use std::f64::consts::SQRT_2;

use cgmath::{InnerSpace, Vector3, Vector4};

use super::{
    angular_radians_to_unit_sphere, circumference_ratio_at_mercator_y, closest_point_on_sphere,
    degrees_per_pixel, elevate_surface_point, globe_circumference_pixels, globe_distance_pixels,
    globe_radius_pixels, globe_zoom_adjustment, horizon_plane_to_circle, interpolate_lat_lon,
    lat_lon_bearing_from_orientation, lat_lon_to_unit_sphere, mercator_to_angular_radians,
    orientation_from_lat_lon_bearing, ray_sphere_intersection, unit_sphere_to_lat_lon,
    EARTH_RADIUS_METERS,
};

#[test]
fn globe_preset_transitions_between_zoom_eleven_and_twelve() {
    assert_eq!(super::transition_for_zoom(9.0), 1.0);
    assert_eq!(super::transition_for_zoom(10.0), 1.0);
    assert_eq!(super::transition_for_zoom(11.0), 1.0);
    assert_eq!(super::transition_for_zoom(11.5), 0.5);
    assert_eq!(super::transition_for_zoom(12.0), 0.0);
    assert_eq!(super::transition_for_zoom(16.0), 0.0);
    assert_eq!(super::transition_for_zoom(f64::NAN), 0.0);
}
use crate::coords::LatLon;

fn assert_close(left: f64, right: f64) {
    assert!((left - right).abs() < 1e-10, "{left} != {right}");
}

fn assert_vector_close(left: Vector3<f64>, right: Vector3<f64>) {
    assert_close(left.x, right.x);
    assert_close(left.y, right.y);
    assert_close(left.z, right.z);
}

#[test]
fn globe_pixel_size_matches_reference_values() {
    assert_close(globe_circumference_pixels(1.0, 0.0), 1.0);
    assert_close(globe_circumference_pixels(1.0, 60.0), 2.0);
    assert_close(globe_radius_pixels(2.0 * std::f64::consts::PI, 0.0), 1.0);
    assert_close(degrees_per_pixel(1.0, 0.0), 360.0);
}

#[test]
fn globe_distance_matches_reference_values() {
    assert_close(
        globe_distance_pixels(1.0, 0.0, LatLon::new(0.0, 0.0), LatLon::new(0.0, 90.0)),
        0.25,
    );
    assert_close(
        globe_distance_pixels(1.0, 0.0, LatLon::new(-45.0, 0.0), LatLon::new(45.0, 0.0)),
        0.25,
    );
    assert_close(
        globe_distance_pixels(1.0, 0.0, LatLon::new(0.0, 0.0), LatLon::new(45.0, 45.0)),
        1.0 / 6.0,
    );
}

#[test]
fn mercator_and_angular_coordinates_use_globe_axes() {
    let (longitude, latitude) = mercator_to_angular_radians(0.5, 0.5);
    assert_close(longitude, 0.0);
    assert_close(latitude, 0.0);
    assert_vector_close(
        angular_radians_to_unit_sphere(longitude, latitude),
        Vector3::new(0.0, 0.0, 1.0),
    );
}

#[test]
fn geographic_coordinates_round_trip_across_sphere() {
    for location in [
        LatLon::new(0.0, 0.0),
        LatLon::new(45.0, 90.0),
        LatLon::new(-60.0, -135.0),
        LatLon::new(90.0, 42.0),
    ] {
        let round_trip = unit_sphere_to_lat_lon(lat_lon_to_unit_sphere(location));
        assert_close(round_trip.latitude, location.latitude);
        if location.latitude.abs() < 90.0 {
            assert_close(round_trip.longitude, location.longitude);
        } else {
            assert_close(round_trip.longitude, 0.0);
        }
    }
}

#[test]
fn orientations_round_trip_center_and_bearing() {
    let center = LatLon::new(37.5, -122.25);
    let state = lat_lon_bearing_from_orientation(orientation_from_lat_lon_bearing(center, 28.0));

    assert_close(state.center.latitude, center.latitude);
    assert_close(state.center.longitude, center.longitude);
    assert_close(state.bearing, 28.0);
}

#[test]
fn horizon_plane_produces_circle_on_unit_sphere() {
    let circle = horizon_plane_to_circle(Vector4::new(0.0, 0.0, 1.0, -0.5));

    assert_vector_close(circle.center, Vector3::new(0.0, 0.0, 0.5));
    assert_close(circle.radius, 3.0_f64.sqrt() / 2.0);
}

#[test]
fn closest_point_is_radial_from_sphere_center() {
    let point = closest_point_on_sphere(
        Vector3::new(1.0, 1.0, 0.0),
        2.0,
        Vector3::new(2.0, 2.0, 0.0),
    )
    .expect("point outside the sphere center has a radial direction");

    assert_vector_close(point, Vector3::new(1.0 + SQRT_2, 1.0 + SQRT_2, 0.0));
    assert_eq!(
        closest_point_on_sphere(
            Vector3::new(1.0, 1.0, 1.0),
            1.0,
            Vector3::new(1.0, 1.0, 1.0)
        ),
        None
    );
}

#[test]
fn zoom_adjustment_matches_reference_values() {
    assert_close(globe_zoom_adjustment(0.0, 60.0), -1.0);
    assert_close(globe_zoom_adjustment(60.0, 0.0), 1.0);
}

#[test]
fn globe_interpolation_preserves_endpoints() {
    let start = LatLon::new(-40.0, 10.0);
    let midpoint = interpolate_lat_lon(start, 80.0, 100.0, 0.5);
    let end = interpolate_lat_lon(start, 80.0, 100.0, 1.0);

    assert_close(midpoint.latitude, 10.0);
    assert!(midpoint.longitude > 10.0 && midpoint.longitude < 90.0);
    assert_close(end.latitude, 60.0);
    assert_close(end.longitude, 90.0);
}

#[test]
fn elevation_extends_radially() {
    let elevated = elevate_surface_point(Vector3::unit_z(), EARTH_RADIUS_METERS);
    assert_vector_close(elevated, Vector3::new(0.0, 0.0, 2.0));
}

#[test]
fn circumference_ratio_is_symmetric_about_equator() {
    assert_close(circumference_ratio_at_mercator_y(0.5), 1.0);
    assert_close(
        circumference_ratio_at_mercator_y(0.25),
        circumference_ratio_at_mercator_y(0.75),
    );
}

#[test]
fn ray_sphere_intersection_handles_hit_tangent_and_miss() {
    let hit = ray_sphere_intersection(
        Vector3::new(0.0, 0.0, 2.0),
        Vector3::new(0.0, 0.0, -1.0),
        1.0,
    )
    .expect("ray should cross the sphere");
    assert_close(hit.t_min, 1.0);
    assert_close(hit.t_max, 3.0);

    let tangent = ray_sphere_intersection(
        Vector3::new(1.0, 0.0, 2.0),
        Vector3::new(0.0, 0.0, -1.0),
        1.0,
    )
    .expect("ray should touch the sphere");
    assert_close(tangent.t_min, 2.0);
    assert_close(tangent.t_max, 2.0);

    assert_eq!(
        ray_sphere_intersection(
            Vector3::new(2.0, 0.0, 2.0),
            Vector3::new(0.0, 0.0, -1.0),
            1.0,
        ),
        None
    );
}

#[test]
fn generated_surface_vectors_are_normalized() {
    let point = lat_lon_to_unit_sphere(LatLon::new(23.0, 47.0));
    assert_close(point.magnitude2(), 1.0);
}
