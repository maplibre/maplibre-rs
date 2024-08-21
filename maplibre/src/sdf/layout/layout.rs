use crate::sdf::glyph::GlyphDependencies;
use crate::sdf::MapMode;
use std::collections::BTreeSet;

// TODO
struct OverscaledTileID;
struct LayerTypeInfo;
struct ImageDependencies;

pub struct BucketParameters {
    tileID: OverscaledTileID,
    mode: MapMode,
    pixelRatio: f64,
    layerType: LayerTypeInfo,
}

pub struct LayoutParameters<'a> {
    bucketParameters: &'a BucketParameters,
    glyphDependencies: &'a GlyphDependencies,
    imageDependencies: &'a ImageDependencies,
    availableImages: &'a BTreeSet<String>,
}
