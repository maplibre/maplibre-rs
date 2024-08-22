use crate::sdf::geometry_tile_data::{FeatureType, GeometryTileFeature, Identifier, Value};
use crate::sdf::style_types::expression;
use crate::sdf::tagged_string::TaggedString;
use geo_types::GeometryCollection;
use std::cmp::Ordering;

pub struct SymbolFeature {
    pub feature: Box<dyn GeometryTileFeature>,
    pub geometry: GeometryCollection,
    pub formattedText: Option<TaggedString>,
    pub icon: Option<expression::Image>,
    pub sortKey: f64,
    pub index: usize,
    pub allowsVerticalWritingMode: bool,
}

impl PartialEq<Self> for SymbolFeature {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl PartialOrd for SymbolFeature {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.sortKey.partial_cmp(&other.sortKey)
    }
}

impl GeometryTileFeature for SymbolFeature {
    fn getType(&self) -> FeatureType {
        self.feature.getType()
    }
    fn getValue(&self, key: &String) -> Option<&Value> {
        self.feature.getValue(key)
    }
    fn getProperties(&self) -> &serde_json::Value {
        self.feature.getProperties()
    }
    fn getID(&self) -> Identifier {
        self.feature.getID()
    }
    fn getGeometries(&self) -> &GeometryCollection {
        self.feature.getGeometries()
    }
}
impl SymbolFeature {
    fn new(feature: Box<dyn GeometryTileFeature>) -> Self {
        Self {
            geometry: feature.getGeometries().clone(), // we need a mutable copy of the geometry for mergeLines()
            feature: feature, // we need a mutable copy of the geometry for mergeLines(),
            formattedText: None,
            icon: None,
            sortKey: 0.0,
            index: 0,
            allowsVerticalWritingMode: false,
        }
    }
}
