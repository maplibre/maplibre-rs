use crate::euclid::Point2D;
use std::ops::Index;

use crate::sdf::TileSpace;

pub type GeometryCoordinate = Point2D<i16, TileSpace>;

pub struct GeometryCoordinates(pub Vec<GeometryCoordinate>);
impl GeometryCoordinates {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}
impl Index<usize> for GeometryCoordinates {
    type Output = GeometryCoordinate;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
