use lyon::geom::Box2D;
use crate::sdf::grid_index::Circle;
use crate::sdf::Point;

pub struct CollisionFeature; // TODO

pub struct CollisionBox {
    // the box is centered around the anchor point
    anchor: Point<f64>,

    // the offset of the box from the label's anchor point.
    // TODO: might be needed for #13526
    // Point<float> offset;

    // distances to the edges from the anchor
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,

    signedDistanceFromAnchor: f64,
}

pub enum ProjectedCollisionBox {
    Circle(Circle<f64>),
    Box(Box2D<f64>)
}