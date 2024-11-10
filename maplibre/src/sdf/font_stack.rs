//! Translated from https://github.com/maplibre/maplibre-native/blob/4add9ea/src/mbgl/util/font_stack.cpp

use std::collections::BTreeSet;

use crate::{sdf::util::hash_combine, style::layer::StyleLayer};

// An array of font names
pub type FontStack = Vec<String>;
pub type FontStackHash = u64;

pub struct FontStackHasher;

impl FontStackHasher {
    pub fn new(fontStack: &FontStack) -> u64 {
        let mut seed = 0;
        for font in fontStack {
            hash_combine(&mut seed, font);
        }
        return seed;
    }
}

pub fn fontStackToString(fontStack: &FontStack) -> String {
    return fontStack.join(",");
}

/// Statically evaluate layer properties to determine what font stacks are used.
pub fn fontStacks(layers: &Vec<StyleLayer>) -> BTreeSet<FontStack> {
    let mut result = BTreeSet::new();
    for layer in layers {
        populateFontStack(layer, &mut result);
    }

    return result;
}

pub(crate) fn populateFontStack(layer: &StyleLayer, stack: &mut BTreeSet<FontStack>) {
    todo!()
}
