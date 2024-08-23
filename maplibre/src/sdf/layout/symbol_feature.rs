use crate::sdf::geometry_tile_data::{FeatureType, Identifier, Value};
use crate::sdf::style_types::expression;
use crate::sdf::tagged_string::TaggedString;
use geo_types::GeometryCollection;
use std::cmp::Ordering;

#[derive(Clone)]
pub struct SymbolGeometryTileFeature {
    pub feature: Box<SymbolGeometryTileFeature>,
    pub geometry: GeometryCollection,
    pub formattedText: Option<TaggedString>,
    pub icon: Option<expression::Image>,
    pub sortKey: f64,
    pub index: usize,
    pub allowsVerticalWritingMode: bool,
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
        self.feature.getType()
    }
    pub fn getValue(&self, key: &String) -> Option<&Value> {
        self.feature.getValue(key)
    }
    pub fn getProperties(&self) -> &serde_json::Value {
        self.feature.getProperties()
    }
    pub fn getID(&self) -> Identifier {
        self.feature.getID()
    }
    pub fn getGeometries(&self) -> &GeometryCollection {
        self.feature.getGeometries()
    }
}

impl SymbolGeometryTileFeature {
    fn new(feature: Box<SymbolGeometryTileFeature>) -> Self {
        Self {
            geometry: feature.geometry.clone(), // we need a mutable copy of the geometry for mergeLines()
            feature: feature, // we need a mutable copy of the geometry for mergeLines(),
            formattedText: None,
            icon: None,
            sortKey: 0.0,
            index: 0,
            allowsVerticalWritingMode: false,
        }
    }
}
