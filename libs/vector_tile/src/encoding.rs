use std::collections::HashMap;

use crate::geometry::{
    Geometry, GeometryLineString, GeometryPoint, GeometryPolygon, MultiPoint, Point,
};
use crate::protos::vector_tile::{
    Tile as ProtoTile, Tile_Feature as ProtoFeature, Tile_GeomType as ProtoGeomType, Tile_GeomType,
    Tile_Layer as ProtoLayer, Tile_Value as ProtoValue,
};
use crate::tile::{Feature, Layer, PropertyValue, Tile};

pub trait Decode<T> {
    fn decode(self) -> T;
}

/// Decode a PropertyValue
impl Decode<PropertyValue> for ProtoValue {
    fn decode(self) -> PropertyValue {
        if self.has_bool_value() {
            PropertyValue::BoolValue(self.get_bool_value())
        } else if self.has_string_value() {
            PropertyValue::StringValue(String::from(self.get_string_value()))
        } else if self.has_float_value() {
            PropertyValue::FloatValue(self.get_float_value())
        } else if self.has_int_value() {
            PropertyValue::IntValue(self.get_int_value())
        } else if self.has_sint_value() {
            PropertyValue::SIntValue(self.get_sint_value())
        } else if self.has_uint_value() {
            PropertyValue::UIntValue(self.get_uint_value())
        } else if self.has_double_value() {
            PropertyValue::DoubleValue(self.get_double_value())
        } else {
            PropertyValue::Unknown
        }
    }
}

/// Decode a list of PropertyValues
impl Decode<Vec<PropertyValue>> for Vec<ProtoValue> {
    fn decode(self) -> Vec<PropertyValue> {
        self.into_iter().map(|value| value.decode()).collect()
    }
}

const CMD_MOVE_TO: u32 = 1;
const CMD_LINE_TO: u32 = 2;
const CMD_CLOSE_PATH: u32 = 7;

const CMD_MOVE_TO_PARAMETERS: usize = 2;
const CMD_LINE_TO_PARAMETERS: usize = 2;
const CMD_CLOSE_PATH_PARAMETERS: usize = 0;

trait ZigZag {
    /// Encodes a value to zigzag
    fn zigzag(self) -> i32;
    /// Decodes a value from zigzag encoding
    fn zagzig(self) -> i32;
}

impl ZigZag for u32 {
    fn zigzag(self) -> i32 {
        ((self << 1) ^ (self >> 31)) as i32
    }

    fn zagzig(self) -> i32 {
        ((self >> 1) as i32 ^ (-((self & 1) as i32)))
    }
}

impl Decode<GeometryPoint> for Vec<u32> {
    fn decode(self) -> GeometryPoint {
        let mut points = vec![];
        let mut i = 0;

        while i < self.len() - 1 {
            let command = self[i] & 0x7;

            if command != CMD_MOVE_TO {
                // FIXME: ERROR
            }

            let count = (self[i] >> 3) as usize;
            i += 1;

            for parameter in 0..count {
                points.push(Point::new(self[i + parameter].zagzig(), self[i + parameter + 1].zagzig()));
            }

            i += count * CMD_MOVE_TO_PARAMETERS;
        }

        if points.len() == 1 {
            GeometryPoint::Point(points.remove(0))
        } else if points.len() > 1 {
            GeometryPoint::MultiPoint(MultiPoint::new(points))
        } else {
            GeometryPoint::Point(Point::new(0, 0)); // point is at the origin
        }
    }
}

impl Decode<GeometryLineString> for Vec<u32> {
    fn decode(self) -> GeometryLineString {


        i += count * match command {
            CMD_MOVE_TO => CMD_MOVE_TO_PARAMETERS,
            CMD_LINE_TO => CMD_LINE_TO_PARAMETERS,
            CMD_CLOSE_PATH => CMD_CLOSE_PATH_PARAMETERS,
            _ => 0,
        };
    }
}

impl Decode<GeometryPolygon> for Vec<u32> {
    fn decode(self) -> GeometryPolygon {
        GeometryPolygon::Unknown
    }
}

/// Decode a Geometry
impl Decode<Geometry> for ProtoFeature {
    fn decode(self) -> Geometry {
        match &self.get_field_type() {
            Tile_GeomType::UNKNOWN => Geometry::Unknown,
            Tile_GeomType::POINT => Geometry::GeometryPoint(self.geometry.decode()),
            Tile_GeomType::LINESTRING => Geometry::GeometryLineString(self.geometry.decode()),
            Tile_GeomType::POLYGON => Geometry::GeometryPolygon(self.geometry.decode()),
        }
    }
}

/// Decode a Feature
impl Decode<Feature> for (&ProtoLayer, ProtoFeature) {
    fn decode(self) -> Feature {
        let (layer, feature) = self;

        let mut properties = HashMap::new();

        for chunk in feature.tags.chunks(2) {
            let key = chunk[0];
            let value = chunk[1];

            let keys = &layer.keys;
            if let Some(actualKey) = keys.get(key as usize) {
                let values = &layer.values;
                if let Some(actualValue) = values.get(value as usize) {
                    properties.insert(actualKey.clone(), actualValue.clone().decode());
                }
            }
        }
        let geometry = feature.clone().decode(); // FIXME: Inefficient clone

        Feature::new(feature, geometry, properties)
    }
}

/// Decode a Layer
impl Decode<Layer> for ProtoLayer {
    fn decode(mut self) -> Layer {
        // FIXME: Order of features is changed here
        let mut features = Vec::new();

        while let Some(feature) = self.features.pop() {
            features.push((&self, feature).decode())
        }

        Layer::new(self, features)
    }
}

/// Decode a whole Tile
impl Decode<Tile> for ProtoTile {
    fn decode(mut self) -> Tile {
        // FIXME: Order of layers is changed here
        let mut layers = Vec::new();

        while let Some(layer) = self.layers.pop() {
            layers.push(layer.decode())
        }

        Tile::new(self, layers)
    }
}
