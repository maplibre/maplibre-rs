use crate::sdf::image::{ImageContent, ImageStretches};
use geo_types::Rect;
use std::collections::HashMap;

// TODO structs
struct Image;
struct ImageManager;
struct ImageMap;
struct ImageVersionMap;
struct PremultipliedImage;

pub struct ImagePosition {
    pub pixelRatio: f64,
    pub paddedRect: Rect<u16>,
    pub version: u32,
    pub stretchX: ImageStretches,
    pub stretchY: ImageStretches,
    pub content: Option<ImageContent>,
}
impl ImagePosition {
    pub const padding: u16 = 1;

    pub fn tl(&self) -> [u16; 2] {
        return [
            (self.paddedRect.min().x + Self::padding) as u16,
            (self.paddedRect.min().y + Self::padding) as u16,
        ];
    }

    pub fn br(&self) -> [u16; 2] {
        return [
            (self.paddedRect.min().x + self.paddedRect.width() - Self::padding) as u16,
            (self.paddedRect.min().y + self.paddedRect.height() - Self::padding) as u16,
        ];
    }

    pub fn tlbr(&self) -> [u16; 4] {
        let _tl = self.tl();
        let _br = self.br();
        return [_tl[0], _tl[1], _br[0], _br[1]];
    }

    pub fn displaySize(&self) -> [f64; 2] {
        return [
            (self.paddedRect.width() - Self::padding * 2) as f64 / self.pixelRatio,
            (self.paddedRect.height() - Self::padding * 2) as f64 / self.pixelRatio,
        ];
    }
}

pub type ImagePositions = HashMap<String, ImagePosition>;

struct ImagePatch {
    image: Image,
    paddedRect: Rect<u16>,
}

impl ImagePatch {}

struct ImageAtlas {
    image: PremultipliedImage,
    iconPositions: ImagePositions,
    patternPositions: ImagePositions,
}
impl ImageAtlas {
    pub fn getImagePatchesAndUpdateVersions(image_manager: &ImageManager) -> Vec<ImagePatch> {
        todo!()
    }
}

pub fn makeImageAtlas(
    image_map_a: &ImageMap,
    image_map_b: &ImageMap,
    versionMap: &ImageVersionMap,
) -> ImageAtlas {
    todo!()
}
