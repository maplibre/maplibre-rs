//! Vector tile layer drawing utilities.

use std::collections::HashMap;

use cint::{Alpha, EncodedSrgb};
use csscolorparser::Color;
use serde::{Deserialize, Serialize};

use crate::style::raster::RasterLayer;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BackgroundPaint {
    #[serde(rename = "background-color")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<Color>,
    // TODO a lot
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FillPaint {
    #[serde(rename = "fill-color")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_color: Option<Color>,
    // TODO a lot
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LinePaint {
    #[serde(rename = "line-color")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_color: Option<Color>,
    // TODO a lot
}

/// The different types of paints.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "paint")]
pub enum LayerPaint {
    #[serde(rename = "background")]
    Background(BackgroundPaint),
    #[serde(rename = "line")]
    Line(LinePaint),
    #[serde(rename = "fill")]
    Fill(FillPaint),
    #[serde(rename = "raster")]
    Raster(RasterLayer),
}

impl LayerPaint {
    pub fn get_color(&self) -> Option<Alpha<EncodedSrgb<f32>>> {
        match self {
            LayerPaint::Background(paint) => paint
                .background_color
                .as_ref()
                .map(|color| color.clone().into()),
            LayerPaint::Line(paint) => paint.line_color.as_ref().map(|color| color.clone().into()),
            LayerPaint::Fill(paint) => paint.fill_color.as_ref().map(|color| color.clone().into()),
            LayerPaint::Raster(_) => None,
        }
    }
}

/// Stores all the styles for a specific layer.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StyleLayer {
    #[serde(skip)]
    pub index: u32, // FIXME: How is this initialized?
    pub id: String,
    // TODO filter
    // TODO layout
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxzoom: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minzoom: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub paint: Option<LayerPaint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_layer: Option<String>,
}

impl Default for StyleLayer {
    fn default() -> Self {
        Self {
            index: 0,
            id: "id".to_string(),
            maxzoom: None,
            minzoom: None,
            metadata: None,
            paint: None,
            source: None,
            source_layer: Some("does not exist".to_string()),
        }
    }
}
