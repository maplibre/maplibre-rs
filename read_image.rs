use std::env;

fn main() {
    let actual_path = "render-tests/src/tests/fill-color/default/actual.png";
    let expected_path = "render-tests/src/tests/fill-color/default/expected.png";

    let actual = image::open(actual_path).unwrap().into_rgba8();
    let expected = image::open(expected_path).unwrap().into_rgba8();

    let mut actual_colors = std::collections::HashSet::new();
    for p in actual.pixels() {
        actual_colors.insert(p.0);
    }
    
    let mut expected_colors = std::collections::HashSet::new();
    for p in expected.pixels() {
        expected_colors.insert(p.0);
    }

    println!("Actual colors: {:?}", actual_colors);
    println!("Expected colors: {:?}", expected_colors);
}
