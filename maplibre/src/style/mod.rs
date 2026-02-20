//! Vector tile format styling.

use std::collections::HashMap;

pub use cint::*;
use serde::{Deserialize, Serialize};

use crate::style::{layer::StyleLayer, source::Source};

pub mod layer;
pub mod source;

/// Stores the style for a multi-layered map.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Style {
    pub version: u16,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub sources: HashMap<String, Source>,
    pub layers: Vec<StyleLayer>,
    pub center: Option<[f64; 2]>, // TODO: Use LatLon type here
    pub zoom: Option<f64>,
    pub pitch: Option<f64>,
}

/// Default style for https://openmaptiles.org/schema/
impl Default for Style {
    fn default() -> Self {
        let mut style: Style = serde_json::from_str(include_str!("../../res/demotiles.json"))
            .expect("Failed to parse default demotiles.json style");

        // Ensure layers have sequential Z-indices
        for (i, layer) in style.layers.iter_mut().enumerate() {
            layer.index = i as u32;
        }

        style
    }
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

        let _style: Style = serde_json::from_str(style_json_str).unwrap();
    }
}
