use crate::sdf::glyph::GlyphDependencies;
use crate::sdf::image::ImageDependencies;
use crate::sdf::{MapMode, OverscaledTileID};
use std::collections::BTreeSet;

#[derive(Clone)]
pub struct LayerTypeInfo;

#[derive(Clone)]
pub struct BucketParameters {
    pub tileID: OverscaledTileID,
    pub mode: MapMode,
    pub pixelRatio: f64,
    pub layerType: LayerTypeInfo,
}

pub struct LayoutParameters<'a> {
    pub bucketParameters: &'a mut BucketParameters,
    pub glyphDependencies: &'a mut GlyphDependencies,
    pub imageDependencies: &'a mut ImageDependencies,
    pub availableImages: &'a mut BTreeSet<String>,
}
