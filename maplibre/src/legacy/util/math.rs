//! Translated functions from https://github.com/maplibre/maplibre-native/blob/4add9ea/
//! Likely to be replaced by more generic functions.

use std::f64::consts::PI;

use crate::euclid::{Point2D, Vector2D};

/// maplibre/maplibre-native#4add9ea original name: rotate
pub fn rotate<U>(a: &Vector2D<f64, U>, angle: f64) -> Vector2D<f64, U> {
    let cos = angle.cos();
    let sin = angle.sin();
    let x = cos * a.x - sin * a.y;
    let y = sin * a.x + cos * a.y;
    Vector2D::new(x, y)
}

/**
 * @brief Converts degrees to radians
 *
 * @param deg Degrees as float.
 * @return Radians as float.
 */
/// maplibre/maplibre-native#4add9ea original name: deg2radf
pub fn deg2radf(deg: f64) -> f64 {
    deg * PI / 180.0
}

/// maplibre/maplibre-native#4add9ea original name: perp
pub fn perp<U>(a: &Vector2D<f64, U>) -> Vector2D<f64, U> {
    Vector2D::new(-a.y, a.x)
}

pub trait MinMax<T> {
    /// maplibre/maplibre-native#4add9ea original name: max_value
    fn max_value(self) -> T;
    /// maplibre/maplibre-native#4add9ea original name: min_value
    fn min_value(self) -> T;
}

impl MinMax<f64> for [f64; 4] {
    /// maplibre/maplibre-native#4add9ea original name: max_value
    fn max_value(self) -> f64 {
        *self
            .iter()
            .max_by(|a, b| a.total_cmp(b))
            .expect("array is not empty")
    }

    /// maplibre/maplibre-native#4add9ea original name: min_value
    fn min_value(self) -> f64 {
        *self
            .iter()
            .min_by(|a, b| a.total_cmp(b))
            .expect("array is not empty")
    }
}

/// maplibre/maplibre-native#4add9ea original name: convert_point_f64
pub fn convert_point_f64<U>(point: &Point2D<i16, U>) -> Point2D<f64, U> {
    Point2D::new(point.x as f64, point.y as f64)
}

/// maplibre/maplibre-native#4add9ea original name: convert_point_i16
pub fn convert_point_i16<U>(point: &Point2D<f64, U>) -> Point2D<i16, U> {
    Point2D::new(point.x as i16, point.y as i16)
}
