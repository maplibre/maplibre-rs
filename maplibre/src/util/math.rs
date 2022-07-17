use std::{cmp::Ordering, fmt};

use cgmath::{
    ulps_eq, BaseFloat, BaseNum, EuclideanSpace, InnerSpace, Point2, Point3, Vector3, Zero,
};

/// A 3-dimensional plane formed from the equation: `A*x + B*y + C*z - D = 0`.
///
/// # Fields
///
/// - `n`: a unit vector representing the normal of the plane where:
///   - `n.x`: corresponds to `A` in the plane equation
///   - `n.y`: corresponds to `B` in the plane equation
///   - `n.z`: corresponds to `C` in the plane equation
/// - `d`: the distance value, corresponding to `D` in the plane equation
///
/// # Notes
///
/// The `A*x + B*y + C*z - D = 0` form is preferred over the other common
/// alternative, `A*x + B*y + C*z + D = 0`, because it tends to avoid
/// superfluous negations (see _Real Time Collision Detection_, p. 55).
///
/// Copied from: https://github.com/rustgd/collision-rs
pub struct Plane<S> {
    /// Plane normal
    pub n: Vector3<S>,
    /// Plane distance value
    pub d: S,
}

impl<S: BaseFloat> Plane<S> {
    /// Construct a plane from a normal vector and a scalar distance. The
    /// plane will be perpendicular to `n`, and `d` units offset from the
    /// origin.
    pub fn new(n: Vector3<S>, d: S) -> Plane<S> {
        Plane { n, d }
    }

    /// Constructs a plane that passes through the the three points `a`, `b` and `c`
    pub fn from_points(a: Point3<S>, b: Point3<S>, c: Point3<S>) -> Option<Plane<S>> {
        // create two vectors that run parallel to the plane
        let v0 = b - a;
        let v1 = c - a;

        // find the normal vector that is perpendicular to v1 and v2
        let n = v0.cross(v1);
        if ulps_eq!(n, &Vector3::zero()) {
            None
        } else {
            // compute the normal and the distance to the plane
            let n = n.normalize();
            let d = -a.dot(n);

            Some(Plane::new(n, d))
        }
    }

    /// Construct a plane from a point and a normal vector.
    /// The plane will contain the point `p` and be perpendicular to `n`.
    pub fn from_point_normal(p: Point3<S>, n: Vector3<S>) -> Plane<S> {
        Plane { n, d: p.dot(n) }
    }

    fn intersection_distance_ray(
        &self,
        ray_origin: &Vector3<S>,
        ray_direction: &Vector3<S>,
    ) -> Option<S> {
        let vd: S =
            self.n.x * ray_direction.x + self.n.y * ray_direction.y + self.n.z * ray_direction.z;
        if vd == S::zero() {
            return None;
        }
        let t: S =
            -(self.n.x * ray_origin.x + self.n.y * ray_origin.y + self.n.z * ray_origin.z + self.d)
                / vd;

        Some(t)
    }

    /// Returns unsorted intersection points with an Aabb3
    /// Adopted from: https://www.asawicki.info/news_1428_finding_polygon_of_plane-aabb_intersection
    /// Inspired by: https://godotengine.org/qa/54688/camera-frustum-intersection-with-plane
    pub fn intersection_points_aabb3(&self, aabb: &Aabb3<S>) -> Vec<Vector3<S>> {
        let mut out_points: Vec<Vector3<S>> = Vec::new();
        let aabb_min: Vector3<S> = aabb.min.to_vec();
        let aabb_max: Vector3<S> = aabb.max.to_vec();

        // Test edges along X axis, pointing right.
        let mut dir: Vector3<S> = Vector3::new(aabb_max.x - aabb_min.x, S::zero(), S::zero());
        let mut orig = aabb_min;
        if let Some(t) = self.intersection_distance_ray(&orig, &dir) {
            if t >= S::zero() && t <= S::one() {
                out_points.push(orig + dir * t);
            }
        }

        orig = Vector3::new(aabb_min.x, aabb_max.y, aabb_min.z);
        if let Some(t) = self.intersection_distance_ray(&orig, &dir) {
            if t >= S::zero() && t <= S::one() {
                out_points.push(orig + dir * t);
            }
        }

        orig = Vector3::new(aabb_min.x, aabb_min.y, aabb_max.z);
        if let Some(t) = self.intersection_distance_ray(&orig, &dir) {
            if t >= S::zero() && t <= S::one() {
                out_points.push(orig + dir * t);
            }
        }

        orig = Vector3::new(aabb_min.x, aabb_max.y, aabb_max.z);
        if let Some(t) = self.intersection_distance_ray(&orig, &dir) {
            if t >= S::zero() && t <= S::one() {
                out_points.push(orig + dir * t);
            }
        }

        // Test edges along Y axis, pointing up.
        dir = Vector3::new(S::zero(), aabb_max.y - aabb_min.y, S::zero());
        orig = Vector3::new(aabb_min.x, aabb_min.y, aabb_min.z);
        if let Some(t) = self.intersection_distance_ray(&orig, &dir) {
            if t >= S::zero() && t <= S::one() {
                out_points.push(orig + dir * t);
            }
        }

        orig = Vector3::new(aabb_max.x, aabb_min.y, aabb_min.z);
        if let Some(t) = self.intersection_distance_ray(&orig, &dir) {
            if t >= S::zero() && t <= S::one() {
                out_points.push(orig + dir * t);
            }
        }

        orig = Vector3::new(aabb_min.x, aabb_min.y, aabb_max.z);
        if let Some(t) = self.intersection_distance_ray(&orig, &dir) {
            if t >= S::zero() && t <= S::one() {
                out_points.push(orig + dir * t);
            }
        }

        orig = Vector3::new(aabb_max.x, aabb_min.y, aabb_max.z);
        if let Some(t) = self.intersection_distance_ray(&orig, &dir) {
            if t >= S::zero() && t <= S::one() {
                out_points.push(orig + dir * t);
            }
        }

        // Test edges along Z axis, pointing forward.
        dir = Vector3::new(S::zero(), S::zero(), aabb_max.z - aabb_min.z);
        orig = Vector3::new(aabb_min.x, aabb_min.y, aabb_min.z);
        if let Some(t) = self.intersection_distance_ray(&orig, &dir) {
            if t >= S::zero() && t <= S::one() {
                out_points.push(orig + dir * t);
            }
        }

        orig = Vector3::new(aabb_max.x, aabb_min.y, aabb_min.z);
        if let Some(t) = self.intersection_distance_ray(&orig, &dir) {
            if t >= S::zero() && t <= S::one() {
                out_points.push(orig + dir * t);
            }
        }

        orig = Vector3::new(aabb_min.x, aabb_max.y, aabb_min.z);
        if let Some(t) = self.intersection_distance_ray(&orig, &dir) {
            if t >= S::zero() && t <= S::one() {
                out_points.push(orig + dir * t);
            }
        }

        orig = Vector3::new(aabb_max.x, aabb_max.y, aabb_min.z);
        if let Some(t) = self.intersection_distance_ray(&orig, &dir) {
            if t >= S::zero() && t <= S::one() {
                out_points.push(orig + dir * t);
            }
        }

        out_points
    }

    pub fn intersection_polygon_aabb3(&self, aabb: &Aabb3<S>) -> Vec<Vector3<S>> {
        let mut points = self.intersection_points_aabb3(aabb);

        if points.is_empty() {
            return points;
        };

        let plane_normal = Vector3::new(self.n.x, self.n.y, self.n.z);
        let origin = points[0];

        points.sort_by(|a, b| {
            let cmp = (a - origin).cross(b - origin).dot(plane_normal);
            if cmp < S::zero() {
                Ordering::Less
            } else if cmp == S::zero() {
                Ordering::Equal
            } else {
                Ordering::Greater
            }
        });

        points
    }
}

impl<S: BaseFloat> fmt::Debug for Plane<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}x + {:?}y + {:?}z - {:?} = 0",
            self.n.x, self.n.y, self.n.z, self.d
        )
    }
}

pub(crate) fn min<S: PartialOrd + Copy>(lhs: S, rhs: S) -> S {
    match lhs.partial_cmp(&rhs) {
        Some(Ordering::Less) | Some(Ordering::Equal) | None => lhs,
        _ => rhs,
    }
}

pub(crate) fn max<S: PartialOrd + Copy>(lhs: S, rhs: S) -> S {
    match lhs.partial_cmp(&rhs) {
        Some(Ordering::Greater) | Some(Ordering::Equal) | None => lhs,
        _ => rhs,
    }
}

/// A two-dimensional AABB, aka a rectangle.
pub struct Aabb2<S> {
    /// Minimum point of the AABB
    pub min: Point2<S>,
    /// Maximum point of the AABB
    pub max: Point2<S>,
}

impl<S: BaseNum> Aabb2<S> {
    /// Construct a new axis-aligned bounding box from two points.
    #[inline]
    pub fn new(p1: Point2<S>, p2: Point2<S>) -> Aabb2<S> {
        Aabb2 {
            min: Point2::new(min(p1.x, p2.x), min(p1.y, p2.y)),
            max: Point2::new(max(p1.x, p2.x), max(p1.y, p2.y)),
        }
    }

    /// Compute corners.
    #[inline]
    pub fn to_corners(&self) -> [Point2<S>; 4] {
        [
            self.min,
            Point2::new(self.max.x, self.min.y),
            Point2::new(self.min.x, self.max.y),
            self.max,
        ]
    }
}

impl<S: BaseNum> fmt::Debug for Aabb2<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?} - {:?}]", self.min, self.max)
    }
}

/// A three-dimensional AABB, aka a rectangular prism.
pub struct Aabb3<S> {
    /// Minimum point of the AABB
    pub min: Point3<S>,
    /// Maximum point of the AABB
    pub max: Point3<S>,
}

impl<S: BaseNum> Aabb3<S> {
    /// Construct a new axis-aligned bounding box from two points.
    #[inline]
    pub fn new(p1: Point3<S>, p2: Point3<S>) -> Aabb3<S> {
        Aabb3 {
            min: Point3::new(min(p1.x, p2.x), min(p1.y, p2.y), min(p1.z, p2.z)),
            max: Point3::new(max(p1.x, p2.x), max(p1.y, p2.y), max(p1.z, p2.z)),
        }
    }

    /// Compute corners.
    #[inline]
    pub fn to_corners(&self) -> [Point3<S>; 8] {
        [
            self.min,
            Point3::new(self.max.x, self.min.y, self.min.z),
            Point3::new(self.min.x, self.max.y, self.min.z),
            Point3::new(self.max.x, self.max.y, self.min.z),
            Point3::new(self.min.x, self.min.y, self.max.z),
            Point3::new(self.max.x, self.min.y, self.max.z),
            Point3::new(self.min.x, self.max.y, self.max.z),
            self.max,
        ]
    }
}

impl<S: BaseNum> fmt::Debug for Aabb3<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?} - {:?}]", self.min, self.max)
    }
}

pub fn bounds_from_points<P, T>(points: impl Iterator<Item = P>) -> Option<([T; 2], [T; 2])>
where
    P: Into<[T; 2]>,
    T: PartialOrd + Copy,
{
    let mut min: Option<[T; 2]> = None;
    let mut max: Option<[T; 2]> = None;

    for point in points {
        let [x, y] = point.into();

        if let Some([min_x, min_y]) = &mut min {
            if x < *min_x {
                *min_x = x;
            }
            if y < *min_y {
                *min_y = y;
            }
        } else {
            min = Some([x, y])
        }

        if let Some([max_x, max_y]) = &mut max {
            if x > *max_x {
                *max_x = x;
            }
            if y > *max_y {
                *max_y = y;
            }
        } else {
            max = Some([x, y])
        }
    }

    if let (Some(min), Some(max)) = (min, max) {
        Some((min, max))
    } else {
        None
    }
}

/// A wrapper type that enables ordering floats. This is a work around for the famous "rust float
/// ordering" problem. By using it, you acknowledge that sorting NaN is undefined according to spec.
/// This implementation treats NaN as the "smallest" float.
#[derive(Debug, Copy, Clone, PartialOrd)]
pub struct FloatOrd(pub f32);

impl PartialEq for FloatOrd {
    fn eq(&self, other: &Self) -> bool {
        if self.0.is_nan() && other.0.is_nan() {
            true
        } else {
            self.0 == other.0
        }
    }
}

impl Eq for FloatOrd {}

#[allow(clippy::derive_ord_xor_partial_ord)]
impl Ord for FloatOrd {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap_or_else(|| {
            if self.0.is_nan() && !other.0.is_nan() {
                Ordering::Less
            } else if !self.0.is_nan() && other.0.is_nan() {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        })
    }
}

pub const fn div_away(lhs: i32, rhs: i32) -> i32 {
    if rhs < 0 {
        panic!("rhs must be positive")
    }

    if lhs < 0 {
        div_floor(lhs, rhs)
    } else {
        div_ceil(lhs, rhs)
    }
}

pub const fn div_ceil(lhs: i32, rhs: i32) -> i32 {
    let d = lhs / rhs;
    let r = lhs % rhs;
    if (r > 0 && rhs > 0) || (r < 0 && rhs < 0) {
        d + 1
    } else {
        d
    }
}

pub const fn div_floor(lhs: i32, rhs: i32) -> i32 {
    let d = lhs / rhs;
    let r = lhs % rhs;
    if (r > 0 && rhs < 0) || (r < 0 && rhs > 0) {
        d - 1
    } else {
        d
    }
}

#[cfg(test)]
mod tests {
    use crate::{coords::EXTENT_SINT, util::math::div_ceil};

    #[test]
    pub fn test_div_floor() {
        assert_eq!(div_ceil(7000, EXTENT_SINT), 2);
        assert_eq!(div_ceil(-7000, EXTENT_SINT), -1);
    }
}
