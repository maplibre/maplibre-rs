//! Default vector tile styles configuration.
// use std::env;

use std::{collections::HashMap, str::FromStr};
use std::{io, path::Path};

use csscolorparser::Color;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use serde_json::Value;

use crate::style::{
    layerdraft::{LayerPaint, LinePaint, StyleLayer},
    sourcedraft::Source,
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
    pub sources: HashMap<String, Source>,
    pub sprite: Option<String>,
    pub glyphs: Option<String>,
    pub projection: Option<Map<String, Value>>,
    pub layers: Vec<StyleLayer>,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub id: Option<String>,
    pub owner: Option<String>,
    pub visibility: Option<String>,
    pub protected: Option<bool>,
    pub draft: Option<bool>,

    #[serde(flatten)]
    pub extra: Map<String, Value>,
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
            sprite: Some("genericmap://sprites/genericmap".to_string()),
            projection: Default::default(),
            visibility: Some("public".to_string()),
            protected: Some(false),
            modified: Some("2021-03-01T00:00:00Z".to_string()),
            owner: Default::default(),
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
    fn load_test() {
        // let dir = env::current_dir().unwrap();
        // println!("{:#?}", dir);
        let reader = load_model("./src/style/test-data/cl4.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
        println!("test works");
    }

    #[test]
    fn load_sample_data_one() {
        let reader = load_model("./src/style/map-styles/test-cj3kbeq.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_two() {
        let reader = load_model("./src/style/map-styles/test-cj7t3i5.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_three() {
        let reader = load_model("./src/style/map-styles/test-cj44mfr.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_four() {
        let reader = load_model("./src/style/map-styles/test-cjcunv5.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_five() {
        let reader = load_model("./src/style/map-styles/test-cjerxnq.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_six() {
        let reader = load_model("./src/style/map-styles/test-cjtep62.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_seven() {
        let reader = load_model("./src/style/map-styles/test-ck4014y.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_eight() {
        let reader = load_model("./src/style/map-styles/test-ckr0svm.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_nine() {
        let reader = load_model("./src/style/map-styles/test-cks9iem.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_ten() {
        let reader = load_model("./src/style/map-styles/test-cks97e1.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_eleven() {
        let reader = load_model("./src/style/map-styles/test-ckshxkp.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_twelve() {
        let reader = load_model("./src/style/map-styles/test-cksjc2n.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_thirteen() {
        let reader = load_model("./src/style/map-styles/test-ckt20wg.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_fourteen() {
        let reader = load_model("./src/style/map-styles/test-cl4bxa8.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_fifteen() {
        let reader = load_model("./src/style/map-styles/test-cl4fnpo.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_sixteen() {
        let reader = load_model("./src/style/map-styles/test-cl4fotj.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_seventeen() {
        let reader = load_model("./src/style/map-styles/test-cl4gxqw.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_eighteen() {
        let reader = load_model("./src/style/map-styles/test-cl4orrp.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_nineteen() {
        let reader = load_model("./src/style/map-styles/test-cl4whef.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_twenty() {
        let reader = load_model("./src/style/map-styles/test-cl4whev.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_twentyone() {
        let reader = load_model("./src/style/map-styles/test-cl4wxue.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_twentytwo() {
        let reader = load_model("./src/style/map-styles/test-dark-v10.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_twentythree() {
        let reader = load_model("./src/style/map-styles/test-light-v10.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_twentyfour() {
        let reader = load_model("./src/style/map-styles/test-navigation-guidan.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_twentyfive() {
        let reader = load_model("./src/style/map-styles/test-navigation-guidance.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_twentysix() {
        let reader = load_model("./src/style/map-styles/test-outdoors-v11.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_twentyseven() {
        let reader = load_model("./src/style/map-styles/test-satellite-st.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_twentyeight() {
        let reader = load_model("./src/style/map-styles/test-satellite-v9.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }

    #[test]
    fn load_sample_data_twentynine() {
        let reader = load_model("./src/style/map-styles/test-streets-v11.json").unwrap();
        let style: crate::style::styledraft::Style = serde_json::from_str(&reader).unwrap();
        println!("{:#?}", style);
    }
}
