use maplibre::style::layer::{
    BackgroundPaint, FillPaint, LayerPaint, LinePaint, RasterPaint, StyleLayer, SymbolPaint,
};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Deserialize, Debug, Clone)]
pub struct StyleLayerDef {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub maxzoom: Option<u8>,
    pub minzoom: Option<u8>,
    pub metadata: Option<HashMap<String, String>>,
    pub source: Option<String>,
    pub source_layer: Option<String>,
    pub paint: Option<Value>,
}

fn main() {
    let bg_json = r#"{
      "id": "bg",
      "type": "background"
    }"#;

    let fill_json = r#"{
      "id": "fill",
      "type": "fill",
      "paint": {
        "fill-antialias": false
      }
    }"#;

    let bg_def: StyleLayerDef = serde_json::from_str(bg_json).unwrap();
    println!("bg def: {:?}", bg_def);

    let fill_def: StyleLayerDef = serde_json::from_str(fill_json).unwrap();
    println!("fill def: {:?}", fill_def);
}
