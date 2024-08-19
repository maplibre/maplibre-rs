use lyon::geom::euclid::{Point2D, UnknownUnit};

pub type Point<T> = Point2D<T, UnknownUnit>;

pub fn convert_point_f64(point: &Point<i16>) -> Point<f64> {
    Point::new(point.x as f64, point.y as f64)
}

pub fn convert_point_i16(point: &Point<f64>) -> Point<i16> {
    Point::new(point.x as i16, point.y as i16)
}

pub struct Anchor {
    pub point: Point<f64>,
    pub angle: f64,
    pub segment: Option<usize>,
}

pub type Anchors = Vec<Anchor>;
