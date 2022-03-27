use crate::geometry::Geometry;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Tile {
    layers: Vec<Layer>,
}

#[derive(Debug, Clone)]
pub struct Layer {
    name: String,
    version: u32,
    features: Vec<Feature>,
    extent: u32,
}

#[derive(Debug, Clone)]
pub struct Feature {
    id: u64,
    geometry: Geometry,
    properties: HashMap<String, PropertyValue>,
}

#[derive(Debug, Clone)]
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
        id: u64,
        geometry: Geometry,
        properties: HashMap<String, PropertyValue>,
    ) -> Self {
        Feature {
            id,
            geometry,
            properties,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn geometry(&self) -> &Geometry {
        &self.geometry
    }
    pub fn properties(&self) -> &HashMap<String, PropertyValue> {
        &self.properties
    }
}

impl Layer {
    pub(crate) fn new(name: String, version: u32, features: Vec<Feature>, extent: u32) -> Self {
        Layer {
            name,
            version,
            features,
            extent,
        }
    }

    pub fn extent(&self) -> u32 {
        self.extent
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn features(&self) -> &Vec<Feature> {
        &self.features
    }
}

impl Tile {
    pub(crate) fn new(layers: Vec<Layer>) -> Self {
        Tile { layers }
    }

    pub fn layers(&self) -> &Vec<Layer> {
        &self.layers
    }
}
