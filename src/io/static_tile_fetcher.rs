use std::concat;
use std::env;

use async_trait::async_trait;
use include_dir::{include_dir, Dir, File};

use crate::coords::TileCoords;
use crate::error::Error;

use super::TileFetcher;

static TILES: Dir = include_dir!("$OUT_DIR/extracted-tiles");

pub struct StaticTileFetcher;

impl StaticTileFetcher {
    pub fn get_source_path() -> &'static str {
        concat!(env!("OUT_DIR"), "/extracted-tiles")
    }
}

#[async_trait(?Send)]
impl TileFetcher for StaticTileFetcher {
    fn new() -> Self {
        Self {}
    }

    async fn fetch_tile(&self, coords: &TileCoords) -> Result<Vec<u8>, Error> {
        self.sync_fetch_tile(coords)
    }

    fn sync_fetch_tile(&self, coords: &TileCoords) -> Result<Vec<u8>, Error> {
        let tile = TILES
            .get_file(format!("{}/{}/{}.{}", coords.z, coords.x, coords.y, "pbf"))
            .ok_or_else(|| Error::File("Failed to load tile from within the binary".to_string()))?;
        Ok(Vec::from(tile.contents())) // TODO: Unnecessary copy
    }
}

#[cfg(test)]
mod tests {
    use crate::io::TileFetcher;

    use super::StaticTileFetcher;

    #[tokio::test]
    async fn test_tiles_available() {
        let fetcher = StaticTileFetcher::new();
        assert!(fetcher.fetch_tile(&(0, 0, 0).into()).await.is_err()); // World overview
        assert!(fetcher
            .fetch_tile(
                &(
                    crate::example::MUNICH_X,
                    crate::example::MUNICH_Y,
                    crate::example::MUNICH_Z
                )
                    .into()
            )
            .await
            .is_ok()); // Maxvorstadt Munich
    }
}
