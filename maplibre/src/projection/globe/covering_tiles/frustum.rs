use cgmath::{InnerSpace, Vector3, Vector4};

use super::{
    super::{camera::GlobeCameraState, covering::GlobeTileBoundingVolume},
    classify_points, Intersection,
};

const CLIP_CORNERS: [[f64; 4]; 8] = [
    [-1.0, 1.0, -1.0, 1.0],
    [1.0, 1.0, -1.0, 1.0],
    [1.0, -1.0, -1.0, 1.0],
    [-1.0, -1.0, -1.0, 1.0],
    [-1.0, 1.0, 1.0, 1.0],
    [1.0, 1.0, 1.0, 1.0],
    [1.0, -1.0, 1.0, 1.0],
    [-1.0, -1.0, 1.0, 1.0],
];

const PLANE_POINT_INDICES: [[usize; 3]; 6] = [
    [6, 5, 4],
    [0, 1, 2],
    [0, 3, 7],
    [2, 1, 5],
    [3, 2, 6],
    [0, 4, 5],
];

pub(super) struct GlobeFrustum {
    points: [Vector3<f64>; 8],
    planes: [Vector4<f64>; 6],
}

impl GlobeFrustum {
    pub(super) fn from_camera(camera: &GlobeCameraState) -> Self {
        let inverse = camera.inverse_view_projection();
        let mut points = std::array::from_fn(|index| {
            let corner = CLIP_CORNERS[index];
            let projected = inverse * Vector4::new(corner[0], corner[1], -corner[2], corner[3]);
            projected.truncate() / projected.w
        });
        adjust_far_plane(&mut points, camera.clipping_plane());
        let planes = std::array::from_fn(|index| plane_from_points(points, index));
        Self { points, planes }
    }

    pub(super) fn intersects(&self, bounds: &GlobeTileBoundingVolume) -> Intersection {
        let mut result = Intersection::Full;
        for plane in self.planes {
            let classification =
                classify_points(&bounds.points, |point| plane.dot(point.extend(1.0)));
            if classification == Intersection::None {
                return Intersection::None;
            }
            if classification == Intersection::Partial {
                result = Intersection::Partial;
            }
        }
        if result == Intersection::Full {
            return result;
        }
        for plane in bounds.planes {
            if classify_points(&self.points, |point| plane.dot(point.extend(1.0)))
                == Intersection::None
            {
                return Intersection::None;
            }
        }
        Intersection::Partial
    }
}

fn plane_from_points(points: [Vector3<f64>; 8], index: usize) -> Vector4<f64> {
    let indices = PLANE_POINT_INDICES[index];
    let first = points[indices[0]] - points[indices[1]];
    let second = points[indices[2]] - points[indices[1]];
    let normal = first.cross(second).normalize();
    normal.extend(-normal.dot(points[indices[1]]))
}

fn adjust_far_plane(points: &mut [Vector3<f64>; 8], horizon: Vector4<f64>) {
    let mut directions = [Vector3::new(0.0, 0.0, 0.0); 4];
    let mut lengths = [0.0; 4];
    let mut maximum_distance: f64 = 0.0;
    for index in 0..4 {
        let ray = points[index] - points[index + 4];
        lengths[index] = ray.magnitude();
        directions[index] = ray / lengths[index];
        maximum_distance = maximum_distance.max(
            ray_plane_distance(points[index + 4], directions[index], horizon)
                .filter(|distance| *distance >= 0.0)
                .unwrap_or(lengths[index]),
        );
    }
    let near_plane = normalized_near_plane(*points);
    if let Some(ideal_distance) = ideal_far_distance(horizon, near_plane) {
        let ideal_ray_length = ideal_distance / directions[0].dot(near_plane.truncate());
        maximum_distance = maximum_distance.min(ideal_ray_length);
    }
    for index in 0..4 {
        let target = maximum_distance.min(lengths[index]);
        points[index] = points[index + 4] + directions[index] * target;
    }
}

fn normalized_near_plane(points: [Vector3<f64>; 8]) -> Vector4<f64> {
    let indices = PLANE_POINT_INDICES[0];
    let first = points[indices[0]] - points[indices[1]];
    let second = points[indices[2]] - points[indices[1]];
    let normal = first.cross(second).normalize();
    normal.extend(-normal.dot(points[indices[0]]))
}

fn ray_plane_distance(
    origin: Vector3<f64>,
    direction: Vector3<f64>,
    plane: Vector4<f64>,
) -> Option<f64> {
    let divisor = plane.truncate().dot(direction);
    if divisor.abs() <= f64::EPSILON {
        return None;
    }
    Some(-plane.dot(origin.extend(1.0)) / divisor)
}

fn ideal_far_distance(horizon: Vector4<f64>, near: Vector4<f64>) -> Option<f64> {
    let horizon_length = horizon.truncate().magnitude();
    let normalized_horizon = horizon / horizon_length;
    let projected_direction = near.truncate()
        - normalized_horizon.truncate() * near.truncate().dot(normalized_horizon.truncate());
    let projected_length = projected_direction.magnitude();
    if projected_length <= f64::EPSILON {
        return None;
    }
    let circle_radius = (1.0 - normalized_horizon.w * normalized_horizon.w).sqrt();
    let circle_center = normalized_horizon.truncate() * -normalized_horizon.w;
    let furthest = circle_center + projected_direction * (circle_radius / projected_length);
    Some(near.dot(furthest.extend(1.0)))
}

#[cfg(test)]
mod tests {
    use cgmath::{InnerSpace, Point2};

    use super::GlobeFrustum;
    use crate::{
        coords::{LatLon, TileCoords, ZoomLevel, TILE_SIZE},
        projection::globe::{
            camera::{GlobeCameraOptions, GlobeCameraState},
            covering::{globe_tile_bounding_volume, TileElevationRange},
        },
    };

    #[test]
    fn pitched_reference_tile_intersects_every_frustum_plane() {
        let camera = GlobeCameraState::new(GlobeCameraOptions {
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
        .expect("reference camera should be valid");
        let frustum = GlobeFrustum::from_camera(&camera);
        let bounds = globe_tile_bounding_volume(
            TileCoords::from((511, 513, ZoomLevel::new(10))),
            TileElevationRange {
                min_meters: 0.0,
                max_meters: 500.0,
            },
        )
        .expect("reference tile bounds should be valid");

        for (index, plane) in frustum.planes.into_iter().enumerate() {
            let passed = bounds
                .points
                .iter()
                .filter(|point| plane.dot(point.extend(1.0)) >= 0.0)
                .count();
            assert!(
                passed > 0,
                "reference tile rejected by frustum plane {index}"
            );
        }
    }
}
