use crate::sdf::glyph::GlyphDependencies;
use crate::sdf::MapMode;
use std::collections::BTreeSet;

// TODO
struct OverscaledTileID;
struct LayerTypeInfo;
struct ImageDependencies;

pub struct BucketParameters {
   pub tileID: OverscaledTileID,
   pub mode: MapMode,
   pub pixelRatio: f64,
   pub layerType: LayerTypeInfo,
}

pub struct LayoutParameters<'a> {
    pub bucketParameters: &'a BucketParameters,
    pub glyphDependencies: &'a GlyphDependencies,
    pub imageDependencies: &'a ImageDependencies,
    pub availableImages: &'a BTreeSet<String>,
}
