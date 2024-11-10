//! Translated from https://github.com/maplibre/maplibre-native/blob/4add9ea/src/mbgl/style/image.cpp

use std::{cmp::Ordering, collections::HashMap};

// TODO
/// maplibre/maplibre-native#4add9ea original name: ImageManager
pub struct ImageManager;
/// maplibre/maplibre-native#4add9ea original name: PremultipliedImage
pub struct PremultipliedImage;

/// maplibre/maplibre-native#4add9ea original name: ImageStretch
pub type ImageStretch = (f64, f64);
/// maplibre/maplibre-native#4add9ea original name: ImageStretches
pub type ImageStretches = Vec<ImageStretch>;

/// maplibre/maplibre-native#4add9ea original name: ImageContent
#[derive(Clone)]
pub struct ImageContent {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

/// maplibre/maplibre-native#4add9ea original name: Image
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
    /// maplibre/maplibre-native#4add9ea original name: eq
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl PartialOrd<Self> for Image {
    /// maplibre/maplibre-native#4add9ea original name: partial_cmp
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

/// maplibre/maplibre-native#4add9ea original name: ImageType
pub enum ImageType {
    Icon,
    Pattern,
}

/// maplibre/maplibre-native#4add9ea original name: ImageMap
pub type ImageMap = HashMap<String, Image>;
/// maplibre/maplibre-native#4add9ea original name: ImageDependencies
pub type ImageDependencies = HashMap<String, ImageType>;
/// maplibre/maplibre-native#4add9ea original name: ImageRequestPair
pub type ImageRequestPair = (ImageDependencies, u64);
/// maplibre/maplibre-native#4add9ea original name: ImageVersionMap
pub type ImageVersionMap = HashMap<String, u32>;
