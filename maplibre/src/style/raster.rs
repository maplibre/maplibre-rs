//! Raster tile layer description

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RasterResampling {
    #[serde(rename = "linear")]
    Linear,
    #[serde(rename = "nearest")]
    Nearest,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RasterLayer {
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

impl Default for RasterLayer {
    fn default() -> Self {
        RasterLayer {
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
