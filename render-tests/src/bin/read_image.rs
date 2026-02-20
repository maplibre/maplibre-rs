use image::GenericImageView;
use std::collections::HashSet;

fn main() {
    let actual_path = "render-tests/src/tests/fill-color/default/actual.png";
    let expected_path = "render-tests/src/tests/fill-color/default/expected.png";

    if let Ok(actual) = image::open(actual_path) {
        let mut actual_colors = HashSet::new();
        for p in actual.pixels() {
            actual_colors.insert(p.2 .0);
        }
        println!("Actual unique colors: {:?}", actual_colors);
        println!("Actual dimensions: {:?}", actual.dimensions());
    } else {
        println!("Could not open actual.png");
    }

    if let Ok(expected) = image::open(expected_path) {
        let mut expected_colors = HashSet::new();
        for p in expected.pixels() {
            expected_colors.insert(p.2 .0);
        }
        println!("Expected unique colors: {:?}", expected_colors);
        println!("Expected dimensions: {:?}", expected.dimensions());
    } else {
        println!("Could not open expected.png");
    }
}
