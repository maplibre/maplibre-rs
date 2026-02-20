//! Vector tile layer drawing utilities.

use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use cint::{Alpha, EncodedSrgb};
use csscolorparser::Color;
use serde::{Deserialize, Serialize};

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
    #[serde(rename = "line-color", skip_serializing_if = "Option::is_none")]
    pub line_color: Option<Color>,
    // TODO a lot
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RasterResampling {
    #[serde(rename = "linear")]
    Linear,
    #[serde(rename = "nearest")]
    Nearest,
}

/// Raster tile layer description
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RasterPaint {
    #[serde(rename = "raster-brightness-max")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raster_brightness_max: Option<f32>,
    #[serde(rename = "raster-brightness-min")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raster_brightness_min: Option<f32>,
    #[serde(rename = "raster-contrast")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raster_contrast: Option<f32>,
    #[serde(rename = "raster-fade-duration")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raster_fade_duration: Option<u32>,
    #[serde(rename = "raster-hue-rotate")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raster_hue_rotate: Option<f32>,
    #[serde(rename = "raster-opacity")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raster_opacity: Option<f32>,
    #[serde(rename = "raster-resampling")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raster_resampling: Option<RasterResampling>,
    #[serde(rename = "raster-saturation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raster_saturation: Option<f32>,
}

impl Default for RasterPaint {
    fn default() -> Self {
        RasterPaint {
            raster_brightness_max: Some(1.0),
            raster_brightness_min: Some(0.0),
            raster_contrast: Some(0.0),
            raster_fade_duration: Some(0),
            raster_hue_rotate: Some(0.0),
            raster_opacity: Some(1.0),
            raster_resampling: Some(RasterResampling::Linear),
            raster_saturation: Some(0.0),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SymbolPaint {
    #[serde(rename = "text-field")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_field: Option<String>,
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
    Raster(RasterPaint),
    #[serde(rename = "symbol")]
    Symbol(SymbolPaint),
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
            LayerPaint::Symbol(_) => None,
        }
    }
}

/// Stores all the styles for a specific layer.
#[derive(Serialize, Debug, Clone)]
pub struct StyleLayer {
    #[serde(skip)]
    pub index: u32, // FIXME: How is this initialized?
    pub id: String, // todo make sure that ids are unique. Styles with non-unique layer ids must not exist
    #[serde(rename = "type")]
    pub type_: String,
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

#[derive(Deserialize)]
struct StyleLayerDef {
    id: String,
    #[serde(rename = "type")]
    type_: String,
    maxzoom: Option<u8>,
    minzoom: Option<u8>,
    metadata: Option<HashMap<String, String>>,
    source: Option<String>,
    source_layer: Option<String>,
    paint: Option<serde_json::Value>,
}

impl<'de> serde::Deserialize<'de> for StyleLayer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let def = StyleLayerDef::deserialize(deserializer)?;

        let paint = if let Some(p) = def.paint {
            match def.type_.as_str() {
                "background" => serde_json::from_value(p).map(LayerPaint::Background).ok(),
                "line" => serde_json::from_value(p).map(LayerPaint::Line).ok(),
                "fill" => serde_json::from_value(p).map(LayerPaint::Fill).ok(),
                "raster" => serde_json::from_value(p).map(LayerPaint::Raster).ok(),
                "symbol" => serde_json::from_value(p).map(LayerPaint::Symbol).ok(),
                _ => None,
            }
        } else {
            None
        };

        Ok(StyleLayer {
            index: 0,
            id: def.id,
            type_: def.type_,
            maxzoom: def.maxzoom,
            minzoom: def.minzoom,
            metadata: def.metadata,
            paint,
            source: def.source,
            source_layer: def.source_layer,
        })
    }
}

impl Eq for StyleLayer {}
impl PartialEq for StyleLayer {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Hash for StyleLayer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl Default for StyleLayer {
    fn default() -> Self {
        Self {
            index: 0,
            id: "id".to_string(),
            type_: "background".to_string(),
            maxzoom: None,
            minzoom: None,
            metadata: None,
            paint: None,
            source: None,
            source_layer: Some("does not exist".to_string()),
        }
    }
}
