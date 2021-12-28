use std::concat;
use std::env;

use include_dir::{include_dir, Dir, File};

static TILES: Dir = include_dir!("$OUT_DIR/extracted-tiles");

static mut TEST: u32 = 0;

pub fn get_source_path() -> &'static str {
    concat!(env!("OUT_DIR"), "/extracted-tiles")
}

pub fn get_tile(x: u32, y: u32, z: u32) -> Option<&'static File<'static>> {
    TILES.get_file(format!("{}/{}/{}.{}", z, x, y, "pbf"))
}

#[cfg(test)]
mod tests {
    use super::get_tile;

    #[test]
    fn test_tiles_available() {
        assert!(get_tile(0, 0, 0).is_none()); // World overview
        assert!(get_tile(2179, 1421, 12).is_some()); // Maxvorstadt Munich
    }
}
