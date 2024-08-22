use crate::sdf::glyph::GlyphDependencies;
use crate::sdf::image::ImageDependencies;
use crate::sdf::layout::symbol_layout::CanonicalTileID;
use crate::sdf::MapMode;
use std::collections::BTreeSet;

// TODO
#[derive(Copy, Clone)]
struct OverscaledTileID {
    pub canonical: CanonicalTileID,
    pub     overscaledZ: u8,
}

impl OverscaledTileID {
    pub fn overscaleFactor(&self) -> u32 {
        return 1 << (self.overscaledZ - self.canonical.z);
    }
}

struct LayerTypeInfo;

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
