use std::fs::File;
use std::io::BufReader;
use crate::encoding::Decode;

use protobuf::Message;

use crate::protos::vector_tile::Tile;

#[test]
fn it_works() {
    let mut f = File::open("libs/vector_tile/test_data/europe.pbf").expect("no file found");
    //let mut f = File::open("test_data/europe.pbf").expect("no file found");
    let mut reader = BufReader::new(f);
    let x = Tile::parse_from_reader(&mut reader).unwrap().decode();
    println!("{:#?}", x);
}
