//! Translated from https://github.com/maplibre/maplibre-native/blob/4add9ea/src/mbgl/renderer/image_atlas.cpp

use std::collections::HashMap;

use crate::{
    euclid::Rect,
    legacy::{
        image::{
            Image, ImageContent, ImageManager, ImageMap, ImageStretches, ImageVersionMap,
            PremultipliedImage,
        },
        TileSpace,
    },
};

/// maplibre/maplibre-native#4add9ea original name: ImagePosition
#[derive(Clone)]
pub struct ImagePosition {
    pub pixelRatio: f64,
    pub paddedRect: Rect<u16, TileSpace>,
    pub version: u32,
    pub stretchX: ImageStretches,
    pub stretchY: ImageStretches,
    pub content: Option<ImageContent>,
}
impl ImagePosition {
    pub const padding: u16 = 1;

    /// maplibre/maplibre-native#4add9ea original name: tl
    pub fn tl(&self) -> [u16; 2] {
        return [
            (self.paddedRect.min().x + Self::padding) as u16,
            (self.paddedRect.min().y + Self::padding) as u16,
        ];
    }

    /// maplibre/maplibre-native#4add9ea original name: br
    pub fn br(&self) -> [u16; 2] {
        return [
            (self.paddedRect.min().x + self.paddedRect.width() - Self::padding) as u16,
            (self.paddedRect.min().y + self.paddedRect.height() - Self::padding) as u16,
        ];
    }

    /// maplibre/maplibre-native#4add9ea original name: tlbr
    pub fn tlbr(&self) -> [u16; 4] {
        let _tl = self.tl();
        let _br = self.br();
        return [_tl[0], _tl[1], _br[0], _br[1]];
    }

    /// maplibre/maplibre-native#4add9ea original name: displaySize
    pub fn displaySize(&self) -> [f64; 2] {
        return [
            (self.paddedRect.width() - Self::padding * 2) as f64 / self.pixelRatio,
            (self.paddedRect.height() - Self::padding * 2) as f64 / self.pixelRatio,
        ];
    }
}

/// maplibre/maplibre-native#4add9ea original name: ImagePositions
pub type ImagePositions = HashMap<String, ImagePosition>;

/// maplibre/maplibre-native#4add9ea original name: ImagePatch
struct ImagePatch {
    image: Image,
    paddedRect: Rect<u16, TileSpace>,
}

impl ImagePatch {}

/// maplibre/maplibre-native#4add9ea original name: ImageAtlas
struct ImageAtlas {
    image: PremultipliedImage,
    iconPositions: ImagePositions,
    patternPositions: ImagePositions,
}
impl ImageAtlas {
    /// maplibre/maplibre-native#4add9ea original name: getImagePatchesAndUpdateVersions
    pub fn getImagePatchesAndUpdateVersions(image_manager: &ImageManager) -> Vec<ImagePatch> {
        todo!()
    }
}

/// maplibre/maplibre-native#4add9ea original name: makeImageAtlas
pub fn makeImageAtlas(
    image_map_a: &ImageMap,
    image_map_b: &ImageMap,
    versionMap: &ImageVersionMap,
) -> ImageAtlas {
    todo!()
}
