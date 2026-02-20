use std::{
    collections::{HashMap, HashSet},
    env,
};

use image::GenericImageView;

fn count_colors(img: &image::DynamicImage) {
    let mut counts: std::collections::HashMap<_, usize> = std::collections::HashMap::new();
    for pixel in img.pixels() {
        *counts.entry(pixel.2 .0).or_default() += 1;
    }
    for (color, count) in counts {
        println!("Color {:?}: {} pixels", color, count);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let actual = image::open(&args[1]).unwrap();
    let expected = image::open(&args[2]).unwrap();

    println!("Actual:");
    count_colors(&actual);

    println!("Expected:");
    count_colors(&expected);
}
