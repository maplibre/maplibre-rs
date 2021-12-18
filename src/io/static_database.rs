use std::concat;
use std::env;

use include_dir::{Dir, File, include_dir};

static TILES: Dir = include_dir!("$OUT_DIR/munich-tiles");

pub fn get_tile_count() -> usize {
    TILES.files().count()
}

pub fn get_tile(x: u32, y: u32, z: u32) -> Option<&'static File<'static>> {
    TILES.get_file(format!("{}/{}/{}.{}", z, x, y, "pbf"))
}

mod tests {
    use crate::io::static_database::{get_tile, get_tile_count};

    #[test]
    fn test_tiles_available() {
        assert!(get_tile(0,0,0).is_some()); // World overview
        assert!(get_tile(2179, 1421,12).is_some()); // Maxvorstadt Munich
    }
}