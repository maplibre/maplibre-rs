//! Translated from https://github.com/maplibre/maplibre-native/blob/4add9ea/src/mbgl/layout/symbol_feature.hpp

use std::cmp::Ordering;

use crate::legacy::{
    geometry_tile_data::{FeatureType, GeometryCollection, Identifier, Value},
    style_types::expression,
    tagged_string::TaggedString,
};

// TODO: Actual feature data with properties
/// maplibre/maplibre-native#4add9ea original name: VectorGeometryTileFeature
#[derive(Clone)]
pub struct VectorGeometryTileFeature {
    pub geometry: GeometryCollection,
}

/// maplibre/maplibre-native#4add9ea original name: SymbolGeometryTileFeature
#[derive(Clone)]
pub struct SymbolGeometryTileFeature {
    feature: Box<VectorGeometryTileFeature>,
    pub geometry: GeometryCollection, // we need a mutable copy of the geometry for mergeLines()
    pub formattedText: Option<TaggedString>,
    pub icon: Option<expression::Image>,
    pub sortKey: f64,
    pub index: usize,
}

impl PartialEq<Self> for SymbolGeometryTileFeature {
    /// maplibre/maplibre-native#4add9ea original name: eq
    fn eq(&self, other: &Self) -> bool {
        self.sortKey.eq(&other.sortKey) // TODO is this correct?
    }
}

impl PartialOrd for SymbolGeometryTileFeature {
    /// maplibre/maplibre-native#4add9ea original name: partial_cmp
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.sortKey.partial_cmp(&other.sortKey)
    }
}

impl SymbolGeometryTileFeature {
    /// maplibre/maplibre-native#4add9ea original name: getType
    pub fn getType(&self) -> FeatureType {
        //  todo!()
        FeatureType::Point
    }
    /// maplibre/maplibre-native#4add9ea original name: getValue
    pub fn getValue(&self, key: &String) -> Option<&Value> {
        todo!()
    }
    /// maplibre/maplibre-native#4add9ea original name: getProperties
    pub fn getProperties(&self) -> &serde_json::Value {
        todo!()
    }
    /// maplibre/maplibre-native#4add9ea original name: getID
    pub fn getID(&self) -> Identifier {
        todo!()
    }
    /// maplibre/maplibre-native#4add9ea original name: getGeometries
    pub fn getGeometries(&self) -> &GeometryCollection {
        todo!()
    }
}

impl SymbolGeometryTileFeature {
    /// maplibre/maplibre-native#4add9ea original name: new
    pub fn new(feature: Box<VectorGeometryTileFeature>) -> Self {
        Self {
            geometry: feature.geometry.clone(), // we need a mutable copy of the geometry for mergeLines()
            feature,
            formattedText: None,
            icon: None,
            sortKey: 0.0,
            index: 0,
        }
    }
}
