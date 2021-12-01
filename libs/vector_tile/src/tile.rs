use std::collections::HashMap;

use crate::geometry::Geometry;
use crate::protos::vector_tile::{
    Tile as ProtoTile, Tile_Feature, Tile_Feature as ProtoFeature, Tile_Layer as ProtoLayer,
};

#[derive(Debug)]
pub struct Tile {
    internal: ProtoTile,
    layers: Vec<Layer>,
}

#[derive(Debug)]
pub struct Layer {
    internal: ProtoLayer,
    features: Vec<Feature>,
}

#[derive(Debug)]
pub struct Feature {
    internal: ProtoFeature,
    geometry: Geometry,
    properties: HashMap<String, PropertyValue>,
}

#[derive(Debug)]
pub enum PropertyValue {
    StringValue(String),
    FloatValue(f32),
    DoubleValue(f64),
    IntValue(i64),
    UIntValue(u64),
    SIntValue(i64),
    BoolValue(bool),
    Unknown,
}

impl Feature {
    pub(crate) fn new(
        internal: ProtoFeature,
        geometry: Geometry,
        properties: HashMap<String, PropertyValue>,
    ) -> Self {
        Feature {
            internal,
            geometry,
            properties,
        }
    }

    pub fn id(&self) -> u64 {
        self.internal.get_id()
    }


    pub fn geometry(&self) -> &Geometry {
        &self.geometry
    }
    pub fn properties(&self) -> &HashMap<String, PropertyValue> {
        &self.properties
    }
}

impl Layer {
    pub(crate) fn new(internal: ProtoLayer, features: Vec<Feature>) -> Self {
        Layer { internal, features }
    }

    pub fn extend(&self) -> u32 {
        self.internal.get_extent()
    }

    pub fn version(&self) -> u32 {
        self.internal.get_version()
    }

    pub fn name(&self) -> &str {
        self.internal.get_name()
    }

    pub fn features(&self) -> &Vec<Feature> {
        &self.features
    }
}

impl Tile {
    pub(crate) fn new(internal: ProtoTile, layers: Vec<Layer>) -> Self {
        Tile { internal, layers }
    }

    pub fn layers(&self) -> &Vec<Layer> {
        &self.layers
    }
}
