use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path};

use protobuf::Message;

use crate::encoding::Decode;
use crate::error::Error;
use crate::protos::vector_tile::Tile as TileProto;
use crate::tile::Tile;

mod encoding;
mod protos;

#[cfg(test)]
mod tests;

pub mod geometry;
pub mod tile;
pub mod grid;
pub mod error;

pub fn parse_tile<P: AsRef<Path>>(path: P) -> Result<Tile, Error> {
    let mut f = File::open(path)?;
    let mut reader = BufReader::new(f);
    parse_tile_reader(&mut reader).into()
}

pub fn parse_tile_reader<B: BufRead>(reader: &mut B) -> Result<Tile, Error> {
   if reader.fill_buf()?.is_empty() {
       return Err(Error::Generic("input must not be empty".to_string()));
   }
    let proto_tile = TileProto::parse_from_reader(reader)?;
    Ok(proto_tile.decode())
}

