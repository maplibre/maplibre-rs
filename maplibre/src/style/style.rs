//! Default vector tile styles configuration.
// use std::env;

use std::{collections::HashMap, io, path::Path, str::FromStr};

use csscolorparser::Color as CssColor;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

// use serde_json::Number;
// use serde_json::json;
use crate::style::{
    layer::{FillPaint, LayerPaint, LinePaint, StyleLayer},
    raster::RasterLayer,
    source::Source,
};

/// Stores the style for a multi-layered map.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Style {
    pub version: u16,
    pub name: String,
    pub metadata: Map<String, Value>,
    pub center: Option<[f64; 2]>,
    // pub center: Option<[f64; 2]>, // TODO: Use LatLon type here
    pub zoom: Option<f64>,
    pub bearing: Option<f64>,
    pub pitch: Option<f64>,
    pub light: Option<Light>,
    pub terrain: Option<Map<String, Value>>,
    pub sources: HashMap<String, Source>,
    pub sprite: Option<String>,
    pub glyphs: Option<String>,
    pub transition: Option<Transition>,

    // TODO is this in the style spec?
    pub projection: Option<Map<String, Value>>,

    pub layers: Vec<StyleLayer>,

    // TODO this is metadata, and it's not in the spec
    pub created: Option<String>,
    pub modified: Option<String>,
    pub id: Option<String>,
    pub owner: Option<String>,
    pub visibility: Option<String>,
    pub protected: Option<bool>,
    pub draft: Option<bool>,

    // to allow for extra fields in the style
    #[serde(flatten)]
    pub extra: Option<Map<String, Value>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Metadata {
    Value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Light {
    #[serde(default)]
    anchor: Anchor,
    #[serde(default)]
    color: Color,
    #[serde(default)]
    intensity: Intensity,
    #[serde(default)]
    position: Position,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Color(String);
impl Default for Color {
    fn default() -> Self {
        Color("#ffffff".parse().unwrap())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Intensity(Vec<f32>);
impl Default for Intensity {
    fn default() -> Self {
        Intensity(vec![1.15, 210.0, 30.0])
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Position(f32);
impl Default for Position {
    fn default() -> Self {
        Position(0.5)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Anchor {
    #[serde(rename = "map")]
    Map,
    #[serde(rename = "viewport")]
    Viewport,
}

impl Default for Anchor {
    fn default() -> Self {
        Anchor::Viewport
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Transition {
    #[serde(default)]
    delay: Delay,
    #[serde(default)]
    duration: Duration,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Delay(i32);
impl Default for Delay {
    fn default() -> Self {
        Delay(0)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Duration(i32);
impl Default for Duration {
    fn default() -> Self {
        Duration(0)
    }
}

impl Default for Style {
    fn default() -> Self {
        Style {
            extra: Default::default(),
            version: 8,
            name: "Default Style".to_string(),
            metadata: Default::default(),
            sources: Default::default(),
            center: Some([46.5197, 6.6323]),
            pitch: Some(0.0),
            zoom: Some(13.0),
            bearing: Some(0.0),
            created: Some("2021-03-01T00:00:00Z".to_string()),
            draft: Some(false),
            glyphs: Some("genericmap://fonts/genericmap/{fontstack}/{range}.pbf".to_string()),
            id: Some("genericmap".to_string()),
            sprite: Some("genericmap://sprites/genericmap".to_string()),
            projection: Default::default(),
            visibility: Some("public".to_string()),
            protected: Some(false),
            modified: Some("2021-03-01T00:00:00Z".to_string()),
            owner: Default::default(),
            light: Some(Light::default()),
            terrain: Default::default(),
            transition: Some(Transition::default()),
            layers: vec![
                StyleLayer {
                    index: 0,
                    id: "park".to_string(),
                    typ: Some("fill".to_string()),
                    maxzoom: None,
                    minzoom: None,
                    metadata: None,
                    paint: Some(LayerPaint::Fill(FillPaint {
                        fill_color: Some(CssColor::from_str("#c8facc").unwrap()),
                    })),
                    source: None,
                    source_layer: Some("park".to_string()),
                },
                StyleLayer {
                    index: 1,
                    id: "landuse".to_string(),
                    typ: Some("fill".to_string()),
                    maxzoom: None,
                    minzoom: None,
                    metadata: None,
                    paint: Some(LayerPaint::Fill(FillPaint {
                        fill_color: Some(CssColor::from_str("#e0dfdf").unwrap()),
                    })),
                    source: None,
                    source_layer: Some("landuse".to_string()),
                },
                StyleLayer {
                    index: 2,
                    id: "landcover".to_string(),
                    typ: Some("fill".to_string()),
                    maxzoom: None,
                    minzoom: None,
                    metadata: None,
                    paint: Some(LayerPaint::Fill(FillPaint {
                        fill_color: Some(CssColor::from_str("#aedfa3").unwrap()),
                    })),
                    source: None,
                    source_layer: Some("landcover".to_string()),
                },
                StyleLayer {
                    index: 3,
                    id: "transportation".to_string(),
                    typ: Some("line".to_string()),
                    maxzoom: None,
                    minzoom: None,
                    metadata: None,
                    paint: Some(LayerPaint::Line(LinePaint {
                        line_color: Some(CssColor::from_str("#ffffff").unwrap()),
                    })),
                    source: None,
                    source_layer: Some("transportation".to_string()),
                },
                StyleLayer {
                    index: 4,
                    id: "building".to_string(),
                    typ: Some("fill".to_string()),
                    maxzoom: None,
                    minzoom: None,
                    metadata: None,
                    paint: Some(LayerPaint::Fill(FillPaint {
                        fill_color: Some(CssColor::from_str("#d9d0c9").unwrap()),
                    })),
                    source: None,
                    source_layer: Some("building".to_string()),
                },
                StyleLayer {
                    index: 4,
                    id: "water".to_string(),
                    typ: Some("fill".to_string()),
                    maxzoom: None,
                    minzoom: None,
                    metadata: None,
                    paint: Some(LayerPaint::Fill(FillPaint {
                        fill_color: Some(CssColor::from_str("#aad3df").unwrap()),
                    })),
                    source: None,
                    source_layer: Some("water".to_string()),
                },
                StyleLayer {
                    index: 6,
                    id: "waterway".to_string(),
                    typ: Some("fill".to_string()),
                    maxzoom: None,
                    minzoom: None,
                    metadata: None,
                    paint: Some(LayerPaint::Fill(FillPaint {
                        fill_color: Some(CssColor::from_str("#aad3df").unwrap()),
                    })),
                    source: None,
                    source_layer: Some("waterway".to_string()),
                },
                StyleLayer {
                    index: 7,
                    id: "boundary".to_string(),
                    typ: Some("line".to_string()),
                    maxzoom: None,
                    minzoom: None,
                    metadata: None,
                    paint: Some(LayerPaint::Line(LinePaint {
                        line_color: Some(CssColor::from_str("black").unwrap()),
                    })),
                    source: None,
                    source_layer: Some("boundary".to_string()),
                },
                StyleLayer {
                    index: 8,
                    id: "raster".to_string(),
                    maxzoom: None,
                    minzoom: None,
                    metadata: None,
                    paint: Some(LayerPaint::Raster(RasterLayer::default())),
                    source: None,
                    source_layer: Some("raster".to_string()),
                    typ: Some("raster".to_string()),
                },
            ],
        }
    }
}

#[derive(Debug)]
pub enum StyleModuleError {
    Io(io::Error),
    Json(serde_json::Error),
}

impl From<io::Error> for StyleModuleError {
    fn from(err: io::Error) -> Self {
        StyleModuleError::Io(err)
    }
}

pub fn load_model<P: AsRef<Path>>(path: P) -> Result<String, StyleModuleError> {
    let file = std::fs::read_to_string(path)?;
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reading() {
        // language=JSON
        let style_json_str = r##"
        {
          "version": 8,
          "name": "Test Style",
          "metadata": {},
          "sources": {
            "openmaptiles": {
              "type": "vector",
              "url": "https://maps.tuerantuer.org/europe_germany/tiles.json"
            }
          },
          "layers": [
            {
              "id": "background",
              "type": "background",
              "paint": {"background-color": "rgb(239,239,239)"}
            },
            {
              "id": "transportation",
              "type": "line",
              "source": "openmaptiles",
              "source-layer": "transportation",
              "paint": {
                "line-color": "#3D3D3D"
              }
            },
            {
              "id": "boundary",
              "type": "line",
              "source": "openmaptiles",
              "source-layer": "boundary",
              "paint": {
                "line-color": "#3D3D3D"
              }
            },
            {
              "id": "building",
              "minzoom": 14,
              "maxzoom": 15,
              "type": "fill",
              "source": "openmaptiles",
              "source-layer": "building",
              "paint": {
                "line-color": "#3D3D3D"
              }
            }
          ]
        }
        "##;

        let thestyle: Style = serde_json::from_str(style_json_str).unwrap();
        println!("{:#?}", thestyle);
    }

    #[test]
    fn default_object() {
        let style = Style::default();
        println!("{:#?}", style);
    }

    #[test]
    fn load_test() {
        let reader = load_model("./src/style/tests/test-data/cl4.json").unwrap();
        let style: crate::style::style::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
        println!("test works");
    }
}
