/*use std::fs::File;
use std::io::BufReader;
use serde_json::Value;*/

fn generate_type_def() -> Option<u32> {
    /*    let f = File::open("style-spec-v8.json").unwrap();
    let mut reader = BufReader::new(f);
    let result = serde_json::from_reader::<_, Value>(&mut reader).unwrap();

    let spec_root = result.as_object()?;
    let version = &spec_root["$version"].as_i64()?;
    let root = &spec_root["$root"].as_object()?;

    for x in spec_root {

    }

    println!("cargo:warning={:?}", version);*/

    Some(5)
}

fn main() {
    generate_type_def();
}
