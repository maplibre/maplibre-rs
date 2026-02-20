use image::GenericImageView;
fn main() {
    let img = image::open("render-tests/src/tests/fill-color/default/expected.png").unwrap();
    println!("Dimensions: {:?}", img.dimensions());
    let mut black = 0;
    let mut white = 0;
    let mut other = 0;
    for pixel in img.pixels() {
        let p = pixel.2;
        if p[0] == 0 && p[1] == 0 && p[2] == 0 { black += 1; }
        else if p[0] == 255 && p[1] == 255 && p[2] == 255 { white += 1; }
        else { other += 1; }
    }
    println!("Black: {}, White: {}, Other: {}", black, white, other);
}
