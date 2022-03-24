use std::concat;
use std::env;

use include_dir::Dir;
use log::error;

use crate::coords::TileCoords;
use crate::error::Error;

#[cfg(static_tiles)]
static TILES: Dir = include_dir!("$OUT_DIR/extracted-tiles");
#[cfg(not(static_tiles))]
static TILES: Dir = Dir::new("/path", &[]);

pub struct StaticTileFetcher;

impl StaticTileFetcher {
    pub fn get_source_path() -> &'static str {
        concat!(env!("OUT_DIR"), "/extracted-tiles")
    }

    pub fn new() -> Self {
        Self {}
    }

    pub async fn fetch_tile(&self, coords: &TileCoords) -> Result<Vec<u8>, Error> {
        self.sync_fetch_tile(coords)
    }

    pub fn sync_fetch_tile(&self, coords: &TileCoords) -> Result<Vec<u8>, Error> {
        if TILES.entries().is_empty() {
            error!(
                "There are not tiles statically embedded in this binary! StaticTileFetcher will \
                not return any tiles!"
            )
        }

        let tile = TILES
            .get_file(format!("{}/{}/{}.{}", coords.z, coords.x, coords.y, "pbf"))
            .ok_or_else(|| Error::File("Failed to load tile from within the binary".to_string()))?;
        Ok(Vec::from(tile.contents())) // TODO: Unnecessary copy
    }
}

#[cfg(test)]
mod tests {
    use style_spec::source::TileAddressingScheme;

    use crate::coords::WorldTileCoords;

    use super::StaticTileFetcher;

    #[tokio::test]
    async fn test_tiles_available() {
        const MUNICH_X: i32 = 17425;
        const MUNICH_Y: i32 = 11365;
        const MUNICH_Z: u8 = 15;

        let fetcher = StaticTileFetcher::new();
        assert!(fetcher.fetch_tile(&(0, 0, 0).into()).await.is_err()); // World overview
        let world_tile: WorldTileCoords = (MUNICH_X, MUNICH_Y, MUNICH_Z).into();
        assert!(fetcher
            .fetch_tile(&world_tile.into_tile(TileAddressingScheme::XYZ))
            .await
            .is_ok()); // Maxvorstadt Munich
    }
}
