use crate::coords::TileCoords;
use crate::error::Error;
use async_trait::async_trait;

pub mod cache;
pub mod static_tile_fetcher;
pub mod web_tile_fetcher;

#[async_trait(?Send)]
pub trait HttpFetcher {
    fn new() -> Self;

    async fn fetch(&self, url: &str) -> Result<Vec<u8>, Error>;
}

#[async_trait(?Send)]
pub trait TileFetcher {
    fn new() -> Self;

    async fn fetch_tile(&self, coords: &TileCoords) -> Result<Vec<u8>, Error>;
    fn sync_fetch_tile(&self, coords: &TileCoords) -> Result<Vec<u8>, Error>;
}
