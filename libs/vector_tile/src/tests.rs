use std::fs::File;
use std::io::{BufReader, Cursor};

use protobuf::Message;

use crate::encoding::Decode;
use crate::grid::{google_mercator, tile_coordinates_bavaria};
use crate::protos::vector_tile::Tile;
use crate::{parse_tile, parse_tile_reader};

#[test]
fn test_parsing_europe_pbf() {
    parse_tile("libs/vector_tile/test_data/europe.pbf");
}

#[test]
fn test_tile_coordinates_bavaria() {
    println!("{:?}", tile_coordinates_bavaria(&google_mercator(), 6));
}

#[test]
fn test_empty_fail() {
    assert!(parse_tile_reader(&mut Cursor::new(&[])).is_err())
}
