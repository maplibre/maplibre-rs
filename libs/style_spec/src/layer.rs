use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct FillPaint {
    #[serde(rename(serialize = "fill-color"))]
    fill_color: Option<String>,
    // TODO a lot
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LinePaint {
    #[serde(rename(serialize = "line-color"))]
    line_color: Option<String>,
    // TODO a lot
}

#[derive(Serialize, Deserialize, Debug)]
pub enum LayerPaint {
    #[serde(rename(serialize = "line"))]
    Line(LinePaint),
    #[serde(rename(serialize = "fill"))]
    Fill(FillPaint),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub struct Layer {
    id: String,
    // TODO filter
    // TODO layout
    maxzoom: Option<u8>,
    minzoom: Option<u8>,
    metadata: HashMap<String, String>,
    paint: Option<LayerPaint>,
    source: Option<String>,
    source_layer: Option<String>,
}
