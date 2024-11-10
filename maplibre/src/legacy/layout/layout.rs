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
    pub tileID: OverscaledTileID,
    pub mode: MapMode,
    pub pixelRatio: f64,
    pub layerType: LayerTypeInfo,
}

/// maplibre/maplibre-native#4add9ea original name: LayoutParameters
pub struct LayoutParameters<'a> {
    pub bucketParameters: &'a mut BucketParameters,
    pub glyphDependencies: &'a mut GlyphDependencies,
    pub imageDependencies: &'a mut ImageDependencies,
    pub availableImages: &'a mut BTreeSet<String>,
}
