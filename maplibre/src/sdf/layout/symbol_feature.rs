use crate::sdf::geometry_tile_data::{FeatureType, GeometryCollection, Identifier, Value};
use crate::sdf::style_types::expression;
use crate::sdf::tagged_string::TaggedString;

use std::cmp::Ordering;

// TODO: Actual feature data with properties
#[derive(Clone)]
pub struct VectorGeometryTileFeature {
    pub geometry: GeometryCollection,
}

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
    fn eq(&self, other: &Self) -> bool {
        self.sortKey.eq(&other.sortKey) // TODO is this correct?
    }
}

impl PartialOrd for SymbolGeometryTileFeature {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.sortKey.partial_cmp(&other.sortKey)
    }
}

impl SymbolGeometryTileFeature {
    pub fn getType(&self) -> FeatureType {
        //  todo!()
        FeatureType::Point
    }
    pub fn getValue(&self, key: &String) -> Option<&Value> {
        todo!()
    }
    pub fn getProperties(&self) -> &serde_json::Value {
        todo!()
    }
    pub fn getID(&self) -> Identifier {
        todo!()
    }
    pub fn getGeometries(&self) -> &GeometryCollection {
        todo!()
    }
}

impl SymbolGeometryTileFeature {
    pub fn new(feature: Box<VectorGeometryTileFeature>) -> Self {
        Self {
            geometry: feature.geometry.clone(), // we need a mutable copy of the geometry for mergeLines()
            feature: feature,
            formattedText: None,
            icon: None,
            sortKey: 0.0,
            index: 0,
        }
    }
}
