//! Default vector tile styles configuration.

use std::{collections::HashMap, str::FromStr};

use csscolorparser::Color;
use serde::{Deserialize, Serialize};

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
    pub metadata: HashMap<String, String>,
    pub sources: HashMap<String, Source>,
    pub layers: Vec<StyleLayer>,
    pub center: Option<[f64; 2]>, // TODO: Use LatLon type here
    pub zoom: Option<f64>,
    pub pitch: Option<f64>,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            version: 8,
            name: "Default Style".to_string(),
            metadata: Default::default(),
            sources: Default::default(),
            center: Some([50.85045, 4.34878]),
            pitch: Some(0.0),
            zoom: Some(13.0),
            layers: vec![
                StyleLayer {
                    index: 0,
                    id: "park".to_string(),
                    maxzoom: None,
                    minzoom: None,
                    metadata: None,
                    paint: Some(LayerPaint::Fill(FillPaint {
                        fill_color: Some(Color::from_str("#c8facc").unwrap()),
                    })),
                    source: None,
                    source_layer: Some("park".to_string()),
                },
                StyleLayer {
                    index: 1,
                    id: "landuse".to_string(),
                    maxzoom: None,
                    minzoom: None,
                    metadata: None,
                    paint: Some(LayerPaint::Fill(FillPaint {
                        fill_color: Some(Color::from_str("#e0dfdf").unwrap()),
                    })),
                    source: None,
                    source_layer: Some("landuse".to_string()),
                },
                StyleLayer {
                    index: 2,
                    id: "landcover".to_string(),
                    maxzoom: None,
                    minzoom: None,
                    metadata: None,
                    paint: Some(LayerPaint::Fill(FillPaint {
                        fill_color: Some(Color::from_str("#aedfa3").unwrap()),
                    })),
                    source: None,
                    source_layer: Some("landcover".to_string()),
                },
                StyleLayer {
                    index: 3,
                    id: "transportation".to_string(),
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
                    maxzoom: None,
                    minzoom: None,
                    metadata: None,
                    paint: Some(LayerPaint::Fill(FillPaint {
                        fill_color: Some(Color::from_str("#d9d0c9").unwrap()),
                    })),
                    source: None,
                    source_layer: Some("building".to_string()),
                },
                StyleLayer {
                    index: 4,
                    id: "water".to_string(),
                    maxzoom: None,
                    minzoom: None,
                    metadata: None,
                    paint: Some(LayerPaint::Fill(FillPaint {
                        fill_color: Some(Color::from_str("#aad3df").unwrap()),
                    })),
                    source: None,
                    source_layer: Some("water".to_string()),
                },
                StyleLayer {
                    index: 6,
                    id: "waterway".to_string(),
                    maxzoom: None,
                    minzoom: None,
                    metadata: None,
                    paint: Some(LayerPaint::Fill(FillPaint {
                        fill_color: Some(Color::from_str("#aad3df").unwrap()),
                    })),
                    source: None,
                    source_layer: Some("waterway".to_string()),
                },
                StyleLayer {
                    index: 7,
                    id: "boundary".to_string(),
                    maxzoom: None,
                    minzoom: None,
                    metadata: None,
                    paint: Some(LayerPaint::Line(LinePaint {
                        line_color: Some(Color::from_str("black").unwrap()),
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
                },
            ],
        }
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
