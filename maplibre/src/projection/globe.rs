//! Globe projection mathematics shared by camera, mesh, and terrain code.

use std::f64::consts::PI;

use cgmath::{InnerSpace, Quaternion, Vector3, Vector4};

use crate::coords::{LatLon, EXTENT};

pub mod camera;
pub mod covering;
pub mod covering_tiles;
pub mod interaction;
pub mod subdivision;
pub mod tile_mesh;

/// Mean Earth radius used to convert elevation in metres to globe radius.
pub const EARTH_RADIUS_METERS: f64 = 6_371_008.8;
/// GL JS implements the `globe` preset as a blend from vertical perspective at zoom 11 to
/// Mercator at zoom 12, and its render goldens at zoom 11 expect a full globe. The style
/// specification's documentation shows the same blend starting at zoom 10; the renderer follows
/// the implementation and its goldens.
const GLOBE_TRANSITION_START_ZOOM: f64 = 11.0;
const GLOBE_TRANSITION_END_ZOOM: f64 = 12.0;

/// Returns the `globe` preset's vertical-perspective weight at a continuous zoom.
pub fn transition_for_zoom(zoom: f64) -> f32 {
    if !zoom.is_finite() {
        return 0.0;
    }
    ((GLOBE_TRANSITION_END_ZOOM - zoom) / (GLOBE_TRANSITION_END_ZOOM - GLOBE_TRANSITION_START_ZOOM))
        .clamp(0.0, 1.0) as f32
}

const MIN_HORIZONTAL_LENGTH: f64 = 1e-6;
const MIN_VECTOR_LENGTH_SQUARED: f64 = 1e-24;

/// Circle containing the visible horizon of a unit sphere.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HorizonCircle {
    /// Center of the horizon circle.
    pub center: Vector3<f64>,
    /// Radius of the horizon circle.
    pub radius: f64,
}

/// Ordered ray parameters where a ray intersects a sphere.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RaySphereIntersection {
    /// First intersection along the ray direction.
    pub t_min: f64,
    /// Second intersection along the ray direction.
    pub t_max: f64,
}

/// Geographic center and bearing represented by a globe orientation.
#[derive(Clone, Copy, Debug)]
pub struct GlobeOrientationState {
    /// Geographic map center.
    pub center: LatLon,
    /// Clockwise bearing in degrees.
    pub bearing: f64,
}

/// Returns the globe radius in pixels while preserving map-center feature scale.
pub fn globe_radius_pixels(world_size: f64, latitude_degrees: f64) -> f64 {
    world_size / (2.0 * PI) / latitude_degrees.to_radians().cos()
}

/// Returns the globe circumference in pixels at the map center latitude.
pub fn globe_circumference_pixels(world_size: f64, latitude_degrees: f64) -> f64 {
    2.0 * PI * globe_radius_pixels(world_size, latitude_degrees)
}

/// Returns the great-circle distance between two locations in screen pixels.
pub fn globe_distance_pixels(
    world_size: f64,
    center_latitude_degrees: f64,
    first: LatLon,
    second: LatLon,
) -> f64 {
    let first_vector = lat_lon_to_unit_sphere(first);
    let second_vector = lat_lon_to_unit_sphere(second);
    let angle = first_vector.dot(second_vector).clamp(-1.0, 1.0).acos();
    let circumference = globe_circumference_pixels(world_size, center_latitude_degrees);
    angle / (2.0 * PI) * circumference
}

/// Converts normalized Web Mercator coordinates to longitude and latitude in radians.
pub fn mercator_to_angular_radians(mercator_x: f64, mercator_y: f64) -> (f64, f64) {
    let longitude = (mercator_x * PI * 2.0 + PI).rem_euclid(PI * 2.0);
    let latitude = 2.0 * (PI - mercator_y * PI * 2.0).exp().atan() - PI * 0.5;
    (longitude, latitude)
}

/// Converts longitude and latitude in radians to a point on the unit sphere.
pub fn angular_radians_to_unit_sphere(longitude: f64, latitude: f64) -> Vector3<f64> {
    let horizontal_length = latitude.cos();
    Vector3::new(
        longitude.sin() * horizontal_length,
        latitude.sin(),
        longitude.cos() * horizontal_length,
    )
}

/// Projects tile-local Web Mercator coordinates onto a unit sphere.
///
/// Tile coordinates use the crate's [`EXTENT`] and XYZ addressing. Positive Y points north and
/// longitude and latitude zero map to positive Z.
pub fn project_tile_coordinates_to_unit_sphere(
    tile_x: u32,
    tile_y: u32,
    zoom: u8,
    in_tile_x: f64,
    in_tile_y: f64,
) -> Vector3<f64> {
    let tile_count = 2_f64.powi(i32::from(zoom));
    let mercator_x = (f64::from(tile_x) + in_tile_x / EXTENT) / tile_count;
    let mercator_y = (f64::from(tile_y) + in_tile_y / EXTENT) / tile_count;
    let longitude = mercator_x * PI * 2.0 + PI;

    // This form preserves equatorial precision on GPUs where subtracting PI / 2 after inverse
    // Web Mercator conversion discards significant float32 mantissa bits.
    let tangent_half_latitude = (PI - mercator_y * PI * 2.0).exp();
    let tangent_half_latitude_squared = tangent_half_latitude * tangent_half_latitude;
    let denominator = tangent_half_latitude_squared + 1.0;
    let sin_latitude = (tangent_half_latitude_squared - 1.0) / denominator;
    let cos_latitude = (2.0 * tangent_half_latitude) / denominator;

    Vector3::new(
        longitude.sin() * cos_latitude,
        sin_latitude,
        longitude.cos() * cos_latitude,
    )
}

/// Converts a geographic location in degrees to a point on the unit sphere.
pub fn lat_lon_to_unit_sphere(location: LatLon) -> Vector3<f64> {
    angular_radians_to_unit_sphere(
        location.longitude.to_radians(),
        location.latitude.to_radians(),
    )
}

/// Converts a normalized unit-sphere point to geographic coordinates in degrees.
pub fn unit_sphere_to_lat_lon(surface: Vector3<f64>) -> LatLon {
    let latitude = surface.y.clamp(-1.0, 1.0).asin().to_degrees();
    let horizontal_length = (surface.x * surface.x + surface.z * surface.z).sqrt();
    let longitude = if horizontal_length > MIN_HORIZONTAL_LENGTH {
        surface.x.atan2(surface.z).to_degrees()
    } else {
        0.0
    };
    LatLon::new(latitude, wrap_longitude(longitude))
}

/// Returns the quaternion representing a geographic center and bearing.
pub fn orientation_from_lat_lon_bearing(center: LatLon, bearing_degrees: f64) -> Quaternion<f64> {
    let half_x = (-center.longitude).to_radians() * 0.5;
    let half_y = (-center.latitude).to_radians() * 0.5;
    let half_z = bearing_degrees.to_radians() * 0.5;
    let (sin_x, cos_x) = half_x.sin_cos();
    let (sin_y, cos_y) = half_y.sin_cos();
    let (sin_z, cos_z) = half_z.sin_cos();

    Quaternion::new(
        cos_x * cos_y * cos_z + sin_x * sin_y * sin_z,
        sin_x * cos_y * cos_z - cos_x * sin_y * sin_z,
        cos_x * sin_y * cos_z + sin_x * cos_y * sin_z,
        cos_x * cos_y * sin_z - sin_x * sin_y * cos_z,
    )
}

/// Converts a globe orientation quaternion to geographic center and bearing.
pub fn lat_lon_bearing_from_orientation(orientation: Quaternion<f64>) -> GlobeOrientationState {
    let x = orientation.v.x;
    let y = orientation.v.y;
    let z = orientation.v.z;
    let w = orientation.s;
    let longitude = -(2.0 * (w * x + y * z)).atan2(1.0 - 2.0 * (x * x + y * y));
    let latitude = -(2.0 * (w * y - z * x)).clamp(-1.0, 1.0).asin();
    let bearing = (2.0 * (w * z + x * y)).atan2(1.0 - 2.0 * (y * y + z * z));

    GlobeOrientationState {
        center: LatLon::new(latitude.to_degrees(), longitude.to_degrees()),
        bearing: bearing.to_degrees(),
    }
}

/// Computes the center and radius of the visible horizon circle of a unit sphere.
pub fn horizon_plane_to_circle(horizon_plane: Vector4<f64>) -> HorizonCircle {
    let center = horizon_plane.truncate() * -horizon_plane.w;
    let radius = (1.0 - horizon_plane.w * horizon_plane.w).max(0.0).sqrt();
    HorizonCircle { center, radius }
}

/// Returns the closest point on a sphere, or `None` when direction is undefined at its center.
pub fn closest_point_on_sphere(
    center: Vector3<f64>,
    radius: f64,
    point: Vector3<f64>,
) -> Option<Vector3<f64>> {
    let offset = point - center;
    let length_squared = offset.magnitude2();
    if length_squared <= MIN_VECTOR_LENGTH_SQUARED {
        return None;
    }
    Some(center + offset * (radius / length_squared.sqrt()))
}

/// Returns the zoom delta required to preserve globe radius across a latitude change.
pub fn globe_zoom_adjustment(old_latitude: f64, new_latitude: f64) -> f64 {
    let old_scale = old_latitude.to_radians().cos();
    let new_scale = new_latitude.to_radians().cos();
    (new_scale / old_scale).log2()
}

/// Returns geographic degrees represented by one pixel at the map center.
pub fn degrees_per_pixel(world_size: f64, latitude_degrees: f64) -> f64 {
    360.0 / globe_circumference_pixels(world_size, latitude_degrees)
}

/// Interpolates globe locations while preserving apparent longitudinal speed.
pub fn interpolate_lat_lon(
    start: LatLon,
    longitude_delta: f64,
    latitude_delta: f64,
    interpolation: f64,
) -> LatLon {
    let latitude = start.latitude + latitude_delta * interpolation;
    let longitude_interpolation = if latitude_delta.abs() > 1.0 {
        curved_longitude_interpolation(start.latitude, latitude_delta, interpolation)
    } else {
        interpolation
    };
    LatLon::new(
        latitude,
        start.longitude + longitude_delta * longitude_interpolation,
    )
}

/// Applies elevation in metres radially to a unit-sphere surface point.
pub fn elevate_surface_point(surface: Vector3<f64>, elevation_meters: f64) -> Vector3<f64> {
    surface * (1.0 + elevation_meters / EARTH_RADIUS_METERS)
}

/// Returns latitude circumference relative to the equator for normalized Mercator Y.
pub fn circumference_ratio_at_mercator_y(mercator_y: f64) -> f64 {
    let tangent_half_latitude = (PI - mercator_y * PI * 2.0).exp();
    2.0 * tangent_half_latitude / (tangent_half_latitude * tangent_half_latitude + 1.0)
}

/// Intersects a normalized ray with a sphere centered at the origin.
pub fn ray_sphere_intersection(
    origin: Vector3<f64>,
    direction: Vector3<f64>,
    radius: f64,
) -> Option<RaySphereIntersection> {
    let origin_dot_direction = origin.dot(direction);
    let perpendicular = origin - direction * origin_dot_direction;
    let discriminant = radius * radius - perpendicular.magnitude2();
    if discriminant < 0.0 {
        return None;
    }

    let square_root = discriminant.sqrt();
    let q = -origin_dot_direction
        + if origin_dot_direction < 0.0 {
            square_root
        } else {
            -square_root
        };
    if q.abs() <= f64::EPSILON {
        let tangent = -origin_dot_direction;
        return Some(RaySphereIntersection {
            t_min: tangent,
            t_max: tangent,
        });
    }

    let first = (origin.magnitude2() - radius * radius) / q;
    let second = q;
    Some(RaySphereIntersection {
        t_min: first.min(second),
        t_max: first.max(second),
    })
}

fn curved_longitude_interpolation(
    start_latitude: f64,
    latitude_delta: f64,
    interpolation: f64,
) -> f64 {
    let end_latitude = start_latitude + latitude_delta;
    let crosses_equator = end_latitude.signum() != start_latitude.signum();
    let sample_start = if crosses_equator {
        -start_latitude.abs()
    } else {
        start_latitude.abs()
    }
    .to_radians();
    let sample_end = end_latitude.abs().to_radians();
    let value_start = integrate_secant(sample_start);
    let value_end = integrate_secant(sample_end);
    let value = integrate_secant(sample_start + interpolation * (sample_end - sample_start));
    (value - value_start) / (value_end - value_start)
}

fn integrate_secant(radians: f64) -> f64 {
    let half = radians * 0.5;
    let (sin, cos) = half.sin_cos();
    (sin + cos).ln() - (cos - sin).ln()
}

fn wrap_longitude(longitude: f64) -> f64 {
    (longitude + 180.0).rem_euclid(360.0) - 180.0
}

#[cfg(test)]
mod tests;
