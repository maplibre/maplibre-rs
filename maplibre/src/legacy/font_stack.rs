//! Translated from https://github.com/maplibre/maplibre-native/blob/4add9ea/src/mbgl/util/font_stack.cpp

use std::collections::BTreeSet;

use crate::{legacy::util::hash_combine, style::layer::StyleLayer};

// An array of font names
/// maplibre/maplibre-native#4add9ea original name: FontStack
pub type FontStack = Vec<String>;
/// maplibre/maplibre-native#4add9ea original name: FontStackHash
pub type FontStackHash = u64;

/// maplibre/maplibre-native#4add9ea original name: FontStackHasher
pub struct FontStackHasher;

impl FontStackHasher {
    /// maplibre/maplibre-native#4add9ea original name: new
    pub fn new(fontStack: &FontStack) -> u64 {
        let mut seed = 0;
        for font in fontStack {
            hash_combine(&mut seed, font);
        }
        seed
    }
}

/// maplibre/maplibre-native#4add9ea original name: fontStackToString
pub fn fontStackToString(fontStack: &FontStack) -> String {
    fontStack.join(",")
}

/// Statically evaluate layer properties to determine what font stacks are used.
/// maplibre/maplibre-native#4add9ea original name: fontStacks
pub fn fontStacks(layers: &Vec<StyleLayer>) -> BTreeSet<FontStack> {
    let mut result = BTreeSet::new();
    for layer in layers {
        populateFontStack(layer, &mut result);
    }

    result
}

/// maplibre/maplibre-native#4add9ea original name: populateFontStack
pub(crate) fn populateFontStack(layer: &StyleLayer, stack: &mut BTreeSet<FontStack>) {
    todo!()
}
