//! Vector tile layer drawing utilities.

use crate::style::raster::RasterLayer;
use cint::{Alpha, EncodedSrgb};
use csscolorparser::Color;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

pub enum PaintType {
    Argb(Alpha<EncodedSrgb<f32>>),
    Raster(RasterLayer),
}

impl From<Alpha<EncodedSrgb<f32>>> for PaintType {
    fn from(argb: Alpha<EncodedSrgb<f32>>) -> Self {
        PaintType::Argb(argb)
    }
}

impl From<Color> for PaintType {
    fn from(color: Color) -> Self {
        PaintType::Argb(color.into())
    }
}

impl From<PaintType> for [f32; 4] {
    fn from(paint_type: PaintType) -> Self {
        match paint_type {
            PaintType::Argb(argb) => argb.into(),
            PaintType::Raster(_) => [0.0, 1.0, 1.0, 1.0],
        }
    }
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
    pub fn get_color(&self) -> Option<PaintType> {
        match self {
            LayerPaint::Background(paint) => paint
                .background_color
                .as_ref()
                .map(|color| color.clone().into()),
            LayerPaint::Line(paint) => paint.line_color.as_ref().map(|color| color.clone().into()),
            LayerPaint::Fill(paint) => paint.fill_color.as_ref().map(|color| color.clone().into()),
            LayerPaint::Raster(raster) => Some(PaintType::Raster(raster.clone())),
        }
    }
}

/// Stores all the styles for a specific layer.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StyleLayer {
    #[serde(skip)]
    pub index: u32,
    pub id: String,
    #[serde(rename = "type")]
    pub typ: String,
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
            typ: "fill".to_string(),
            maxzoom: None,
            minzoom: None,
            metadata: None,
            paint: None,
            source: None,
            source_layer: Some("does not exist".to_string()),
        }
    }
}
