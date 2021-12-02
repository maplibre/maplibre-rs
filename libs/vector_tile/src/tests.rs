use std::fs::File;
use std::io::BufReader;

use protobuf::Message;

use crate::encoding::Decode;
use crate::grid::{GOOGLE_MERCATOR, tile_coordinates_bavaria};
use crate::parse_tile;
use crate::protos::vector_tile::Tile;

#[test]
fn test_parsing_europe_pbf() {
    parse_tile("libs/vector_tile/test_data/europe.pbf");
}

#[test]
fn test_tile_coordinates_bavaria() {
    println!("{:?}", tile_coordinates_bavaria(&GOOGLE_MERCATOR, 6));
}
