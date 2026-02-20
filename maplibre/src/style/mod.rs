//! Vector tile format styling.

use std::collections::HashMap;

pub use cint::*;
use serde::{Deserialize, Serialize};

use crate::style::{layer::StyleLayer, source::Source};

pub mod layer;
pub mod source;

// ----------------------
// Use manual styel
// ----------------------

// use std::{collections::HashMap, str::FromStr};

// pub use cint::*;
// use csscolorparser::Color;
// use serde::{Deserialize, Serialize};

// pub mod layer;
// pub mod source;

// use crate::style::{
//     layer::{
//         BackgroundPaint, FillPaint, LayerPaint, LinePaint, RasterPaint, StyleLayer, StyleProperty,
//         SymbolPaint,
//     },
//     source::Source,
// };

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

        // Style {
        //     version: 8,
        //     name: Some("Default Style".to_string()),
        //     metadata: Default::default(),
        //     sources: Default::default(),
        //     center: Some([50.85045, 4.34878]),
        //     pitch: Some(0.0),
        //     zoom: Some(13.0),
        //     layers: vec![
        //         StyleLayer {
        //             index: 0,
        //             id: "background".to_string(),
        //             type_: "background".to_string(),
        //             filter: None,
        //             maxzoom: None,
        //             minzoom: None,
        //             metadata: None,
        //             paint: Some(LayerPaint::Background(BackgroundPaint {
        //                 background_color: Some(StyleProperty::Constant(
        //                     Color::from_str("#ffffff").unwrap(),
        //                 )),
        //             })),
        //             source: None,
        //             source_layer: None,
        //         },
        //         StyleLayer {
        //             index: 1,
        //             id: "park".to_string(),
        //             type_: "fill".to_string(),
        //             filter: None,
        //             maxzoom: None,
        //             minzoom: None,
        //             metadata: None,
        //             paint: Some(LayerPaint::Fill(FillPaint {
        //                 fill_color: Some(StyleProperty::Constant(
        //                     Color::from_str("#c8facc").unwrap(),
        //                 )),
        //             })),
        //             source: None,
        //             source_layer: Some("park".to_string()),
        //         },
        //         StyleLayer {
        //             index: 2,
        //             id: "landuse".to_string(),
        //             type_: "fill".to_string(),
        //             filter: None,
        //             maxzoom: None,
        //             minzoom: None,
        //             metadata: None,
        //             paint: Some(LayerPaint::Fill(FillPaint {
        //                 fill_color: Some(StyleProperty::Constant(
        //                     Color::from_str("#e0dfdf").unwrap(),
        //                 )),
        //             })),
        //             source: None,
        //             source_layer: Some("landuse".to_string()),
        //         },
        //         StyleLayer {
        //             index: 3,
        //             id: "landcover".to_string(),
        //             type_: "fill".to_string(),
        //             filter: None,
        //             maxzoom: None,
        //             minzoom: None,
        //             metadata: None,
        //             paint: Some(LayerPaint::Fill(FillPaint {
        //                 fill_color: Some(StyleProperty::Constant(
        //                     Color::from_str("#aedfa3").unwrap(),
        //                 )),
        //             })),
        //             source: None,
        //             source_layer: Some("landcover".to_string()),
        //         },
        //         StyleLayer {
        //             index: 4,
        //             id: "transportation".to_string(),
        //             type_: "line".to_string(),
        //             filter: None,
        //             maxzoom: None,
        //             minzoom: None,
        //             metadata: None,
        //             paint: Some(LayerPaint::Line(LinePaint {
        //                 line_color: Some(StyleProperty::Constant(
        //                     Color::from_str("#ffffff").unwrap(),
        //                 )),
        //                 line_width: None,
        //             })),
        //             source: None,
        //             source_layer: Some("transportation".to_string()),
        //         },
        //         StyleLayer {
        //             index: 5,
        //             id: "building".to_string(),
        //             type_: "fill".to_string(),
        //             filter: None,
        //             maxzoom: None,
        //             minzoom: None,
        //             metadata: None,
        //             paint: Some(LayerPaint::Fill(FillPaint {
        //                 fill_color: Some(StyleProperty::Constant(
        //                     Color::from_str("#d9d0c9").unwrap(),
        //                 )),
        //             })),
        //             source: None,
        //             source_layer: Some("building".to_string()),
        //         },
        //         StyleLayer {
        //             index: 6,
        //             id: "water".to_string(),
        //             type_: "fill".to_string(),
        //             filter: None,
        //             maxzoom: None,
        //             minzoom: None,
        //             metadata: None,
        //             paint: Some(LayerPaint::Fill(FillPaint {
        //                 fill_color: Some(StyleProperty::Constant(
        //                     Color::from_str("#aad3df").unwrap(),
        //                 )),
        //             })),
        //             source: None,
        //             source_layer: Some("water".to_string()),
        //         },
        //         StyleLayer {
        //             index: 7,
        //             id: "waterway".to_string(),
        //             type_: "fill".to_string(),
        //             filter: None,
        //             maxzoom: None,
        //             minzoom: None,
        //             metadata: None,
        //             paint: Some(LayerPaint::Fill(FillPaint {
        //                 fill_color: Some(StyleProperty::Constant(
        //                     Color::from_str("#aad3df").unwrap(),
        //                 )),
        //             })),
        //             source: None,
        //             source_layer: Some("waterway".to_string()),
        //         },
        //         StyleLayer {
        //             index: 8,
        //             id: "boundary".to_string(),
        //             type_: "line".to_string(),
        //             filter: None,
        //             maxzoom: None,
        //             minzoom: None,
        //             metadata: None,
        //             paint: Some(LayerPaint::Line(LinePaint {
        //                 line_color: Some(StyleProperty::Constant(
        //                     Color::from_str("black").unwrap(),
        //                 )),
        //                 line_width: None,
        //             })),
        //             source: None,
        //             source_layer: Some("boundary".to_string()),
        //         },
        //         StyleLayer {
        //             index: 9,
        //             id: "raster".to_string(),
        //             type_: "raster".to_string(),
        //             filter: None,
        //             maxzoom: None,
        //             minzoom: None,
        //             metadata: None,
        //             paint: Some(LayerPaint::Raster(RasterPaint::default())),
        //             source: None,
        //             source_layer: None,
        //         },
        //         StyleLayer {
        //             index: 10,
        //             id: "text".to_string(),
        //             type_: "symbol".to_string(),
        //             filter: None,
        //             maxzoom: None,
        //             minzoom: None,
        //             metadata: None,
        //             paint: Some(LayerPaint::Symbol(SymbolPaint {
        //                 text_field: Some("name".to_string()),
        //                 text_size: None,
        //             })),
        //             source: None,
        //             source_layer: Some("place".to_string()),
        //         },
        //         StyleLayer {
        //             index: 11,
        //             id: "transportation_name".to_string(),
        //             type_: "symbol".to_string(),
        //             filter: None,
        //             maxzoom: None,
        //             minzoom: None,
        //             metadata: None,
        //             paint: Some(LayerPaint::Symbol(SymbolPaint {
        //                 text_field: Some("name".to_string()),
        //                 text_size: None,
        //             })),
        //             source: None,
        //             source_layer: Some("transportation_name-disabled".to_string()),
        //         },
        //     ],
        // }
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

    #[test]
    fn test_style_roundtrip_serde() {
        // Test that the default style can serialize and deserialize (required for web worker Input)
        let style = Style::default();
        let json = serde_json::to_string(&style).unwrap();
        let roundtripped: Style = serde_json::from_str(&json).unwrap();
        assert_eq!(style.layers.len(), roundtripped.layers.len());
        for (orig, rt) in style.layers.iter().zip(roundtripped.layers.iter()) {
            assert_eq!(orig.id, rt.id, "layer ids must match after round-trip");
            assert_eq!(
                orig.type_, rt.type_,
                "layer types must match after round-trip for {}",
                orig.id
            );
        }
    }
}
