use crate::sdf::util::hash_combine;
use crate::style::layer::StyleLayer;
use std::collections::{BTreeSet};

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
