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
    pub pixel_ratio: f64,
    pub padded_rect: Rect<u16, TileSpace>,
    pub version: u32,
    pub stretch_x: ImageStretches,
    pub stretch_y: ImageStretches,
    pub content: Option<ImageContent>,
}
impl ImagePosition {
    pub const PADDING: u16 = 1;

    /// maplibre/maplibre-native#4add9ea original name: tl
    pub fn tl(&self) -> [u16; 2] {
        [
            (self.padded_rect.min().x + Self::PADDING),
            (self.padded_rect.min().y + Self::PADDING),
        ]
    }

    /// maplibre/maplibre-native#4add9ea original name: br
    pub fn br(&self) -> [u16; 2] {
        [
            (self.padded_rect.min().x + self.padded_rect.width() - Self::PADDING),
            (self.padded_rect.min().y + self.padded_rect.height() - Self::PADDING),
        ]
    }

    /// maplibre/maplibre-native#4add9ea original name: tlbr
    pub fn tlbr(&self) -> [u16; 4] {
        let _tl = self.tl();
        let _br = self.br();
        [_tl[0], _tl[1], _br[0], _br[1]]
    }

    /// maplibre/maplibre-native#4add9ea original name: displaySize
    pub fn display_size(&self) -> [f64; 2] {
        [
            (self.padded_rect.width() - Self::PADDING * 2) as f64 / self.pixel_ratio,
            (self.padded_rect.height() - Self::PADDING * 2) as f64 / self.pixel_ratio,
        ]
    }
}

/// maplibre/maplibre-native#4add9ea original name: ImagePositions
pub type ImagePositions = HashMap<String, ImagePosition>;

/// maplibre/maplibre-native#4add9ea original name: ImagePatch
pub struct ImagePatch {
    image: Image,
    padded_rect: Rect<u16, TileSpace>,
}

impl ImagePatch {}

/// maplibre/maplibre-native#4add9ea original name: ImageAtlas
pub struct ImageAtlas {
    image: PremultipliedImage,
    icon_positions: ImagePositions,
    pattern_positions: ImagePositions,
}
impl ImageAtlas {
    /// maplibre/maplibre-native#4add9ea original name: getImagePatchesAndUpdateVersions
    pub fn get_image_patches_and_update_versions(image_manager: &ImageManager) -> Vec<ImagePatch> {
        todo!()
    }
}

/// maplibre/maplibre-native#4add9ea original name: makeImageAtlas
pub fn make_image_atlas(
    image_map_a: &ImageMap,
    image_map_b: &ImageMap,
    version_map: &ImageVersionMap,
) -> ImageAtlas {
    todo!()
}
