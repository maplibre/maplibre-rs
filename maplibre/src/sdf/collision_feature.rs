use crate::sdf::feature_index::IndexedSubfeature;
use crate::sdf::grid_index::Circle;
use crate::sdf::Point;
use lyon::geom::Box2D;

pub struct CollisionFeature {
    pub boxes: Vec<CollisionBox>,
    pub indexedFeature: IndexedSubfeature,
    pub alongLine: bool,
}

#[derive(Default, Clone, Copy)]
pub struct CollisionBox {
    // the box is centered around the anchor point
    pub anchor: Point<f64>,

    // the offset of the box from the label's anchor point.
    // TODO: might be needed for #13526
    // Point<float> offset;

    // distances to the edges from the anchor
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,

    pub signedDistanceFromAnchor: f64,
}

#[derive(Clone, Copy)]
pub enum ProjectedCollisionBox {
    Circle(Circle<f64>),
    Box(Box2D<f64>),
}

impl Default for ProjectedCollisionBox {
    fn default() -> Self {
        return Self::Box(Box2D::zero());
    }
}

impl ProjectedCollisionBox {
    pub fn box_(&self) -> &Box2D<f64> {
        match self {
            ProjectedCollisionBox::Circle(_) => panic!("not a box"),
            ProjectedCollisionBox::Box(box_) => box_,
        }
    }

    pub fn circle(&self) -> &Circle<f64> {
        match self {
            ProjectedCollisionBox::Circle(circle) => circle,
            ProjectedCollisionBox::Box(_) => panic!("not a circle"),
        }
    }

    pub fn isBox(&self) -> bool {
        match self {
            ProjectedCollisionBox::Circle(_) => false,
            ProjectedCollisionBox::Box(_) => true,
        }
    }

    pub fn isCircle(&self) -> bool {
        match self {
            ProjectedCollisionBox::Circle(_) => true,
            ProjectedCollisionBox::Box(_) => false,
        }
    }
}
