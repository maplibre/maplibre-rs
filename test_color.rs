use csscolorparser::parse;

fn main() {
    let c = parse("blue").unwrap();
    println!("Blue: [{}, {}, {}, {}]", c.r as f32, c.g as f32, c.b as f32, c.a as f32);
}
