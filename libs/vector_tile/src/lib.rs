use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use protobuf::Message;

use crate::encoding::Decode;
use crate::error::Error;
use crate::protos::vector_tile::Tile as TileProto;
use crate::tile::Tile;

mod encoding;
mod protos;

#[cfg(test)]
mod tests;

pub mod error;
pub mod geometry;
pub mod grid;
pub mod tile;

pub fn parse_tile<P: AsRef<Path>>(path: P) -> Result<Tile, Error> {
    let f = File::open(path)?;
    let mut reader = BufReader::new(f);
    parse_tile_reader(&mut reader).into()
}

pub fn parse_tile_reader<B: BufRead>(reader: &mut B) -> Result<Tile, Error> {
    let proto_tile = TileProto::parse_from_reader(reader)?;
    Ok(proto_tile.decode())
}

pub fn parse_tile_bytes(bytes: &[u8]) -> Result<Tile, Error> {
    let proto_tile = TileProto::parse_from_bytes(bytes)?;
    Ok(proto_tile.decode())
}
