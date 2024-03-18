//! Default vector tile styles configuration.
// use std::env;

use std::{collections::HashMap, str::FromStr};
use std::{io, path::Path};

use csscolorparser::Color;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use serde_json::Value;

use crate::style::{
    layer::{LayerPaint, LinePaint, StyleLayer},
    sourcedraft::Source,
};

/// Stores the style for a multi-layered map.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Style {
    pub version: u16,
    pub name: String,
    pub metadata: Map<String, Value>,

    #[serde(flatten)]
    pub extra: Map<String, Value>,

    pub sources: HashMap<String, Source>,
    pub layers: Vec<StyleLayer>,
    pub center: Option<[f64; 2]>, // TODO: Use LatLon type here
    pub zoom: Option<f64>,
    pub pitch: Option<f64>,
    pub bearing: Option<f64>,
    pub created: Option<String>,
    pub draft: Option<bool>,
    pub glyphs: Option<String>,
    pub id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Metadata {
    Value: String,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            version: 8,
            name: "Default Style".to_string(),
            metadata: Default::default(),
            extra: Default::default(),
            sources: Default::default(),
            center: Some([46.5197, 6.6323]),
            pitch: Some(0.0),
            zoom: Some(13.0),
            bearing: Some(0.0),
            created: Some("2021-03-01T00:00:00Z".to_string()),
            draft: Some(false),
            glyphs: Some("genericmap://fonts/genericmap/{fontstack}/{range}.pbf".to_string()),
            id: Some("genericmap".to_string()),
            layers: vec![
                StyleLayer {
                    index: 0,
                    id: "park".to_string(),
                    typ: "fill".to_string(),
                    maxzoom: None,
                    minzoom: None,
                    metadata: None,
                    paint: Some(LayerPaint::Line(LinePaint {
                        line_color: Some(Color::from_str("#c8facc").unwrap()),
                    })),
                    source: None,
                    source_layer: Some("park".to_string()),
                },
                StyleLayer {
                    index: 1,
                    id: "landuse".to_string(),
                    typ: "fill".to_string(),
                    maxzoom: None,
                    minzoom: None,
                    metadata: None,
                    paint: Some(LayerPaint::Line(LinePaint {
                        line_color: Some(Color::from_str("#e0dfdf").unwrap()),
                    })),
                    source: None,
                    source_layer: Some("landuse".to_string()),
                },
                StyleLayer {
                    index: 2,
                    id: "landcover".to_string(),
                    typ: "fill".to_string(),
                    maxzoom: None,
                    minzoom: None,
                    metadata: None,
                    paint: Some(LayerPaint::Line(LinePaint {
                        line_color: Some(Color::from_str("#aedfa3").unwrap()),
                    })),
                    source: None,
                    source_layer: Some("landcover".to_string()),
                },
                StyleLayer {
                    index: 3,
                    id: "transportation".to_string(),
                    typ: "line".to_string(),
                    maxzoom: None,
                    minzoom: None,
                    metadata: None,
                    paint: Some(LayerPaint::Line(LinePaint {
                        line_color: Some(Color::from_str("#ffffff").unwrap()),
                    })),
                    source: None,
                    source_layer: Some("transportation".to_string()),
                },
                StyleLayer {
                    index: 4,
                    id: "building".to_string(),
                    typ: "fill".to_string(),
                    maxzoom: None,
                    minzoom: None,
                    metadata: None,
                    paint: Some(LayerPaint::Line(LinePaint {
                        line_color: Some(Color::from_str("#d9d0c9").unwrap()),
                    })),
                    source: None,
                    source_layer: Some("building".to_string()),
                },
                StyleLayer {
                    index: 4,
                    id: "water".to_string(),
                    typ: "fill".to_string(),
                    maxzoom: None,
                    minzoom: None,
                    metadata: None,
                    paint: Some(LayerPaint::Line(LinePaint {
                        line_color: Some(Color::from_str("#aad3df").unwrap()),
                    })),
                    source: None,
                    source_layer: Some("water".to_string()),
                },
                StyleLayer {
                    index: 6,
                    id: "waterway".to_string(),
                    typ: "fill".to_string(),
                    maxzoom: None,
                    minzoom: None,
                    metadata: None,
                    paint: Some(LayerPaint::Line(LinePaint {
                        line_color: Some(Color::from_str("#aad3df").unwrap()),
                    })),
                    source: None,
                    source_layer: Some("waterway".to_string()),
                },
                StyleLayer {
                    index: 7,
                    id: "boundary".to_string(),
                    typ: "line".to_string(),
                    maxzoom: None,
                    minzoom: None,
                    metadata: None,
                    paint: Some(LayerPaint::Line(LinePaint {
                        line_color: Some(Color::from_str("black").unwrap()),
                    })),
                    source: None,
                    source_layer: Some("boundary".to_string()),
                },
            ],
        }
    }
}

#[derive(Debug)]
pub enum StyleModuleError {
    Io(std::io::Error),
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
    fn load_sample_data() {
        // let dir = env::current_dir().unwrap();
        // println!("{:#?}", dir);
        let reader = load_model("./src/style/test-data/cl4.json").unwrap();
        // println!("{:#?}", reader);
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
        println!("helloooo");
    }
}
