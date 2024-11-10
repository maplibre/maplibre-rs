//! Translated from https://github.com/maplibre/maplibre-native/blob/4add9ea/src/mbgl/style/image.cpp

use std::{cmp::Ordering, collections::HashMap};

// TODO
pub struct ImageManager;
pub struct PremultipliedImage;

pub type ImageStretch = (f64, f64);
pub type ImageStretches = Vec<ImageStretch>;

#[derive(Clone)]
pub struct ImageContent {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

pub struct Image {
    pub id: String,

    image: PremultipliedImage,

    // Pixel ratio of the sprite image.
    pub pixelRatio: f64,

    // Whether this image should be interpreted as a signed distance field icon.
    pub sdf: bool,

    // Stretch areas of this image.
    pub stretchX: Option<ImageStretches>,
    pub stretchY: Option<ImageStretches>,

    // The space where text can be fit into this image.
    pub content: Option<ImageContent>,
}

impl PartialEq<Self> for Image {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl PartialOrd<Self> for Image {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

pub enum ImageType {
    Icon,
    Pattern,
}

pub type ImageMap = HashMap<String, Image>;
pub type ImageDependencies = HashMap<String, ImageType>;
pub type ImageRequestPair = (ImageDependencies, u64);
pub type ImageVersionMap = HashMap<String, u32>;
