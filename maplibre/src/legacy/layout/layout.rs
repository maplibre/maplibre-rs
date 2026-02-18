//! Translated from https://github.com/maplibre/maplibre-native/blob/4add9ea/src/mbgl/layout/layout.hpp

use std::collections::BTreeSet;

use crate::legacy::{
    glyph::GlyphDependencies, image::ImageDependencies, MapMode, OverscaledTileID,
};

/// maplibre/maplibre-native#4add9ea original name: LayerTypeInfo
#[derive(Clone)]
pub struct LayerTypeInfo;

/// maplibre/maplibre-native#4add9ea original name: BucketParameters
#[derive(Clone)]
pub struct BucketParameters {
    pub tile_id: OverscaledTileID,
    pub mode: MapMode,
    pub pixel_ratio: f64,
    pub layer_type: LayerTypeInfo,
}

/// maplibre/maplibre-native#4add9ea original name: LayoutParameters
pub struct LayoutParameters<'a> {
    pub bucket_parameters: &'a mut BucketParameters,
    pub glyph_dependencies: &'a mut GlyphDependencies,
    pub image_dependencies: &'a mut ImageDependencies,
    pub available_images: &'a mut BTreeSet<String>,
}
