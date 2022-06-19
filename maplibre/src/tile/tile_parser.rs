use geozero::mvt::Tile;
use prost::Message;

#[derive(Default)]
pub struct TileParser;

impl TileParser {
    pub fn parse(data: Box<[u8]>) -> Tile {
        geozero::mvt::Tile::decode(data.as_ref()).expect("failed to load tile")
    }
}
