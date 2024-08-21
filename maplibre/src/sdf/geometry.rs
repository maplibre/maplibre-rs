use crate::euclid::Point2D;
use crate::sdf::TileSpace;

pub mod feature_index;

pub fn convert_point_f64<U>(point: &Point2D<i16, U>) -> Point2D<f64, U> {
    Point2D::new(point.x as f64, point.y as f64)
}

pub fn convert_point_i16<U>(point: &Point2D<f64, U>) -> Point2D<i16, U> {
    Point2D::new(point.x as i16, point.y as i16)
}

pub struct Anchor {
    pub point: Point2D<f64, TileSpace>,
    pub angle: f64,
    pub segment: Option<usize>,
}

pub type Anchors = Vec<Anchor>;
