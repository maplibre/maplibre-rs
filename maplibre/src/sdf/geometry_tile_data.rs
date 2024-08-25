use crate::euclid::Point2D;
use crate::sdf::layout::symbol_feature::SymbolGeometryTileFeature;
use std::ops::Index;

use crate::sdf::TileSpace;

// In maplibre-native GeometryTileFeature are traits/classes and there are impls for symbol, fill, line features etc.
// The same is true for the data objects which might be backed by geojson

pub type GeometryCoordinate = Point2D<i16, TileSpace>;

#[derive(Default, Clone, Debug)]
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

pub type GeometryCollection = Vec<GeometryCoordinates>;

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

#[derive(Clone)]
pub struct SymbolGeometryTileLayer {
    pub name: String,
    pub features: Vec<SymbolGeometryTileFeature>
}
impl SymbolGeometryTileLayer {
    pub fn featureCount(&self) -> usize {
        self.features.len()
    }

    // Returns the feature object at the given position within the layer. The
    // returned feature object may *not* outlive the layer object.
    pub fn getFeature(&self, index: usize) -> Box<SymbolGeometryTileFeature> {
       Box::new(self.features[index].clone())
    }

    pub fn getName(&self) -> &str {
        &self.name
    }
}

#[derive(Clone)]
struct SymbolGeometryTileData;

impl SymbolGeometryTileData {
    pub fn clone(&self) -> Box<SymbolGeometryTileData> {
        todo!()
    }

    // Returns the layer with the given name. The returned layer object *may*
    // outlive the data object.
    pub fn getLayer(&self, name: &str) -> Box<SymbolGeometryTileLayer> {
        todo!()
    }
}
