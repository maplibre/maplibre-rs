//! Static tile fetcher

use std::{concat, env};

#[cfg(static_tiles_found)]
use include_dir::include_dir;
use include_dir::Dir;

use crate::{coords::TileCoords, error::Error};
#[cfg(static_tiles_found)]
static TILES: Dir = include_dir!("$OUT_DIR/extracted-tiles");
#[cfg(not(static_tiles_found))]
static TILES: Dir = Dir::new("/path", &[]);

/// Load PBF files which were statically embedded in the `build.rs`
#[derive(Default)]
pub struct StaticTileFetcher;

impl StaticTileFetcher {
    pub fn get_source_path() -> &'static str {
        concat!(env!("OUT_DIR"), "/extracted-tiles")
    }

    pub fn new() -> Self {
        Self {}
    }

    /// Fetch the tile static file asynchrounously and returns a vector of bytes or a network error if the file
    /// could not be fetched.
    pub async fn fetch_tile(&self, coords: &TileCoords) -> Result<Vec<u8>, Error> {
        self.sync_fetch_tile(coords)
    }

    /// Fetch the tile static file and returns a vector of bytes or a network error if the file
    /// could not be fetched.
    pub fn sync_fetch_tile(&self, coords: &TileCoords) -> Result<Vec<u8>, Error> {
        if TILES.entries().is_empty() {
            panic!(
                "There are not tiles statically embedded in this binary! StaticTileFetcher will \
                not return any tiles!"
            )
        }

        let tile = TILES
            .get_file(format!("{}/{}/{}.{}", coords.z, coords.x, coords.y, "pbf"))
            .ok_or_else(|| {
                Error::Network("Failed to load tile from within the binary".to_string())
            })?;
        Ok(Vec::from(tile.contents()))
    }
}

#[cfg(test)]
mod tests {
    use super::StaticTileFetcher;
    use crate::{coords::WorldTileCoords, style::source::TileAddressingScheme};

    #[cfg(static_tiles_found)]
    #[tokio::test]
    async fn test_tiles_available() {
        const MUNICH_X: i32 = 17425;
        const MUNICH_Y: i32 = 11365;
        const MUNICH_Z: u8 = 15;

        let fetcher = StaticTileFetcher::new();
        assert!(fetcher.fetch_tile(&(0, 0, 0).into()).await.is_err()); // World overview
        let world_tile: WorldTileCoords = (MUNICH_X, MUNICH_Y, MUNICH_Z).into();
        assert!(fetcher
            .fetch_tile(&world_tile.into_tile(TileAddressingScheme::XYZ).unwrap())
            .await
            .is_ok()); // Maxvorstadt Munich
    }
}
