use crate::euclid::Vector2D;
use std::f64::consts::PI;

pub fn rotate<U>(a: &Vector2D<f64, U>, angle: f64) -> Vector2D<f64, U> {
    let cos = angle.cos();
    let sin = angle.sin();
    let x = cos * a.x - sin * a.y;
    let y = sin * a.x + cos * a.y;
    return Vector2D::new(x, y);
}

/**
 * @brief Converts degrees to radians
 *
 * @param deg Degrees as float.
 * @return Radians as float.
 */
pub fn deg2radf(deg: f64) -> f64 {
    return deg * PI / 180.0;
}

pub fn perp<U>(a: &Vector2D<f64, U>) -> Vector2D<f64, U> {
    return Vector2D::new(-a.y, a.x);
}

pub trait MinMax<T> {
    fn max_value(self) -> T;
    fn min_value(self) -> T;
}

impl MinMax<f64> for [f64; 4] {
    fn max_value(self) -> f64 {
        *self
            .iter()
            .max_by(|a, b| a.total_cmp(b))
            .expect("array is not empty")
    }

    fn min_value(self) -> f64 {
        *self
            .iter()
            .min_by(|a, b| a.total_cmp(b))
            .expect("array is not empty")
    }
}
