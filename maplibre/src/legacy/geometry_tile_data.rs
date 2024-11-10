//! Translated from https://github.com/maplibre/maplibre-native/blob/4add9ea/src/mbgl/tile/geometry_tile_data.cpp
//! In maplibre-native GeometryTileFeature are traits/classes and there are impls for symbol, fill, line features etc.
//! The same is true for the data objects which might be backed by geojson

use std::ops::Index;

use crate::{
    euclid::Point2D,
    legacy::{layout::symbol_feature::SymbolGeometryTileFeature, TileSpace},
};

/// maplibre/maplibre-native#4add9ea original name: GeometryCoordinate
pub type GeometryCoordinate = Point2D<i16, TileSpace>;

/// maplibre/maplibre-native#4add9ea original name: GeometryCoordinates(pub
#[derive(Default, Clone, Debug)]
pub struct GeometryCoordinates(pub Vec<GeometryCoordinate>);
impl GeometryCoordinates {
    /// maplibre/maplibre-native#4add9ea original name: len
    pub fn len(&self) -> usize {
        self.0.len()
    }
}
impl Index<usize> for GeometryCoordinates {
    /// maplibre/maplibre-native#4add9ea original name: Output
    type Output = GeometryCoordinate;

    /// maplibre/maplibre-native#4add9ea original name: index
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

/// maplibre/maplibre-native#4add9ea original name: GeometryCollection
pub type GeometryCollection = Vec<GeometryCoordinates>;

// TODO: The following types are not final
/// maplibre/maplibre-native#4add9ea original name: Value
pub type Value = geo_types::Geometry;
/// maplibre/maplibre-native#4add9ea original name: Identifier
pub type Identifier = String;
/// maplibre/maplibre-native#4add9ea original name: PropertyMap
pub type PropertyMap = serde_json::Value;

/// maplibre/maplibre-native#4add9ea original name: FeatureType
#[derive(PartialEq)]
pub enum FeatureType {
    Unknown = 0,
    Point = 1,
    LineString = 2,
    Polygon = 3,
}

/// maplibre/maplibre-native#4add9ea original name: SymbolGeometryTileLayer
#[derive(Clone)]
pub struct SymbolGeometryTileLayer {
    pub name: String,
    pub features: Vec<SymbolGeometryTileFeature>,
}
impl SymbolGeometryTileLayer {
    /// maplibre/maplibre-native#4add9ea original name: featureCount
    pub fn featureCount(&self) -> usize {
        self.features.len()
    }

    // Returns the feature object at the given position within the layer. The
    // returned feature object may *not* outlive the layer object.
    /// maplibre/maplibre-native#4add9ea original name: getFeature
    pub fn getFeature(&self, index: usize) -> Box<SymbolGeometryTileFeature> {
        Box::new(self.features[index].clone())
    }

    /// maplibre/maplibre-native#4add9ea original name: getName
    pub fn getName(&self) -> &str {
        &self.name
    }
}

/// maplibre/maplibre-native#4add9ea original name: SymbolGeometryTileData
#[derive(Clone)]
struct SymbolGeometryTileData;

impl SymbolGeometryTileData {
    /// maplibre/maplibre-native#4add9ea original name: clone
    pub fn clone(&self) -> Box<SymbolGeometryTileData> {
        todo!()
    }

    // Returns the layer with the given name. The returned layer object *may*
    // outlive the data object.
    /// maplibre/maplibre-native#4add9ea original name: getLayer
    pub fn getLayer(&self, name: &str) -> Box<SymbolGeometryTileLayer> {
        todo!()
    }
}
