use std::concat;
use std::env;

use crate::io::TileCoords;
use include_dir::{include_dir, Dir, File};

static TILES: Dir = include_dir!("$OUT_DIR/extracted-tiles");

pub fn get_source_path() -> &'static str {
    concat!(env!("OUT_DIR"), "/extracted-tiles")
}

pub fn get_tile(coords: &TileCoords) -> Option<&'static File<'static>> {
    TILES.get_file(format!("{}/{}/{}.{}", coords.z, coords.x, coords.y, "pbf"))
}

#[cfg(test)]
mod tests {
    use super::get_tile;

    #[test]
    fn test_tiles_available() {
        assert!(get_tile(&(0, 0, 0).into()).is_none()); // World overview
        assert!(get_tile(&(2179, 1421, 12).into()).is_some()); // Maxvorstadt Munich
    }
}
