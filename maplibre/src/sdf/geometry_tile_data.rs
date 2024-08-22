use crate::euclid::Point2D;
use geo_types::GeometryCollection;
use std::ops::Index;

use crate::sdf::TileSpace;

pub type GeometryCoordinate = Point2D<i16, TileSpace>;

#[derive(Default)]
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

// TODO: The following types are not final
pub type Value = geo_types::Geometry;
pub type Identifier = String;
pub type PropertyMap = serde_json::Value;

#[derive(PartialEq)]
pub enum FeatureType {
    Unknown = 0,
    Point = 1,
    LineString = 2,
    Polygon = 3,
}

pub trait GeometryTileFeature {
    fn getType(&self) -> FeatureType;
    fn getValue(&self, key: &String) -> Option<&Value>;
    fn getProperties(&self) -> &PropertyMap;
    fn getID(&self) -> Identifier;
    fn getGeometries(&self) -> &GeometryCollection;
}

pub trait GeometryTileLayer {
    fn featureCount(&self) -> usize;

    // Returns the feature object at the given position within the layer. The
    // returned feature object may *not* outlive the layer object.
    fn getFeature(&self, index: usize) -> Box<dyn GeometryTileFeature>;

    fn getName(&self) -> String;
}

pub trait GeometryTileData {
    fn clone(&self) -> Box<dyn GeometryTileData>;

    // Returns the layer with the given name. The returned layer object *may*
    // outlive the data object.
    fn getLayer(&self, name: &str) -> Box<dyn GeometryTileLayer>;
}
