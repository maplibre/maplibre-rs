use std::collections::HashSet;
use crate::style::layer::StyleLayer;

// An array of font names
pub type FontStack = Vec<String>;
pub type FontStackHash = usize;



pub struct FontStackHasher;

impl FontStackHasher {
    pub fn new(fontStack: &FontStack) -> usize {
        let seed = 0;
        for font in fontStack {
            util::hash_combine(seed, font);
        }
        return seed;
    }
}

pub fn fontStackToString(fontStack : &FontStack) -> String {
    return fontStack.join(",");
}

/// Statically evaluate layer properties to determine what font stacks are used.
pub fn fontStacks(layers: &Vec<StyleLayer>) -> HashSet<FontStack> {
    let mut result = HashSet::new();
    for  layer in layers {
        layer.populateFontStack(&mut result);
    }

    return result;
}