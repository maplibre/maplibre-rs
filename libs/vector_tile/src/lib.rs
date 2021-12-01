use std::fs::File;
use std::io;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use protobuf::Message;

use crate::encoding::Decode;
use crate::protos::vector_tile::Tile as TileProto;
use crate::tile::Tile;

mod encoding;
mod protos;

#[cfg(test)]
mod tests;

pub mod geometry;
pub mod tile;
pub mod grid;

pub fn parse_tile<P: AsRef<Path>>(path: P) -> io::Result<Tile> {
    let mut f = File::open(path)?;
    let mut reader = BufReader::new(f);
    return Ok(parse_tile_reader(&mut reader));
}

pub fn parse_tile_reader(reader: &mut dyn Read) -> Tile {
    let proto_tile = TileProto::parse_from_reader(reader).unwrap();
    return proto_tile.decode();
}

