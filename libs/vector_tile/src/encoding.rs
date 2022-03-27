use std::collections::HashMap;

use crate::geometry::{
    Command, Geometry, GeometryLineString, GeometryPoint, GeometryPolygon, LineTo, MoveTo,
    MultiPoint, Point,
};
use crate::protos::vector_tile::{
    Tile as ProtoTile, Tile_Feature as ProtoFeature, Tile_GeomType, Tile_Layer as ProtoLayer,
    Tile_Value as ProtoValue,
};
use crate::tile::{Feature, Layer, PropertyValue, Tile};

pub trait Decode<T> {
    fn decode(self) -> T;
}

/// Decode a PropertyValue
impl Decode<PropertyValue> for ProtoValue {
    fn decode(mut self) -> PropertyValue {
        if self.has_bool_value() {
            PropertyValue::BoolValue(self.get_bool_value())
        } else if self.has_string_value() {
            PropertyValue::StringValue(self.take_string_value())
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
    /// Decodes a value from zigzag encoding
    fn zagzig(self) -> i32;
}

impl ZigZag for u32 {
    fn zagzig(self) -> i32 {
        ((self >> 1) as i32) ^ (-((self & 1) as i32))
    }
}

impl Decode<GeometryPoint> for Vec<u32> {
    fn decode(self) -> GeometryPoint {
        let mut points = vec![];
        let mut i = 0;

        while i < self.len() {
            let command = self[i] & 0x7;

            if command != CMD_MOVE_TO {
                // FIXME: Not allowed in Points
                panic!("error")
            }

            let count = self[i] >> 3;
            i += 1;

            for _ in 0..count {
                points.push(Point::new(self[i].zagzig(), self[i + 1].zagzig()));

                i += CMD_MOVE_TO_PARAMETERS;
            }
        }

        match points.len() {
            0 => GeometryPoint::Point(Point::new(0, 0)),
            1 => GeometryPoint::Point(points.remove(0)),
            _ => GeometryPoint::MultiPoint(MultiPoint::new(points)),
        }
    }
}

impl Decode<GeometryLineString> for Vec<u32> {
    fn decode(self) -> GeometryLineString {
        let mut commands = Vec::with_capacity(self.len()); // Create vec of maximum size
        let mut i = 0;

        while i < self.len() {
            let command = self[i] & 0x7;

            let count = self[i] >> 3;
            i += 1;

            match command {
                CMD_MOVE_TO => {
                    for _ in 0..count {
                        commands.push(Command::MoveTo(MoveTo {
                            x: self[i].zagzig(),
                            y: self[i + 1].zagzig(),
                        }));
                        i += CMD_MOVE_TO_PARAMETERS;
                    }
                }
                CMD_LINE_TO => {
                    for _ in 0..count {
                        commands.push(Command::LineTo(LineTo {
                            x: self[i].zagzig(),
                            y: self[i + 1].zagzig(),
                        }));
                        i += CMD_LINE_TO_PARAMETERS;
                    }
                }
                CMD_CLOSE_PATH => {
                    // FIXME: Not allowed in LineStrings
                    panic!("error")
                }
                _ => {
                    panic!("error")
                }
            }
        }

        GeometryLineString { commands }
    }
}

impl Decode<GeometryPolygon> for Vec<u32> {
    fn decode(self) -> GeometryPolygon {
        let mut commands = vec![];
        let mut i = 0;

        while i < self.len() {
            let command = self[i] & 0x7;
            let count = self[i] >> 3;

            // parsed command and count => +1
            i += 1;

            match command {
                CMD_MOVE_TO => {
                    for _ in 0..count {
                        commands.push(Command::MoveTo(MoveTo {
                            x: self[i].zagzig(),
                            y: self[i + 1].zagzig(),
                        }));
                        i += CMD_MOVE_TO_PARAMETERS;
                    }
                }
                CMD_LINE_TO => {
                    for _ in 0..count {
                        commands.push(Command::LineTo(LineTo {
                            x: self[i].zagzig(),
                            y: self[i + 1].zagzig(),
                        }));
                        i += CMD_LINE_TO_PARAMETERS;
                    }
                }
                CMD_CLOSE_PATH => {
                    if count != 1 {
                        panic!("error")
                    }
                    commands.push(Command::Close);
                    i += CMD_CLOSE_PATH_PARAMETERS;
                }
                _ => {
                    panic!("error")
                }
            }
        }

        GeometryPolygon { commands }
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
// FIXME: Decoding is very slow right now on development builds of wasm: (Development: 15s, Production: 60ms)
impl Decode<Feature> for (&mut ProtoLayer, ProtoFeature) {
    fn decode(self) -> Feature {
        let (layer, feature) = self;

        let mut properties = HashMap::with_capacity(feature.tags.len());

        let keys = &mut layer.keys;

        for chunk in feature.tags.chunks(2) {
            let key = chunk[0];
            let value = chunk[1];

            if let Some(actual_key) = keys.get(key as usize) {
                let values = &layer.values;
                if let Some(actual_value) = values.get(value as usize) {
                    properties.insert(actual_key.clone(), actual_value.clone().decode());
                }
            }
        }

        let id = feature.get_id();
        let geometry = feature.decode();

        Feature::new(id, geometry, properties)
    }
}

/// Decode a Layer
impl Decode<Layer> for ProtoLayer {
    fn decode(mut self) -> Layer {
        let mut features = Vec::new();

        while let Some(feature) = self.features.pop() {
            features.insert(0, (&mut self, feature).decode())
        }

        Layer::new(
            self.take_name(),
            self.get_version(),
            features,
            self.get_extent(),
        )
    }
}

/// Decode a whole Tile
impl Decode<Tile> for ProtoTile {
    fn decode(mut self) -> Tile {
        let mut layers = Vec::new();

        while let Some(layer) = self.layers.pop() {
            layers.insert(0, layer.decode())
        }

        Tile::new(layers)
    }
}
