//! Handles IO related processing as well as multithreading.

use crate::coords::TileCoords;
use crate::error::Error;
use async_trait::async_trait;

pub mod static_tile_fetcher;
pub mod web_tile_fetcher;
pub mod worker_loop;

pub struct HttpFetcherConfig {
    /// Under which path should we cache requests.
    pub cache_path: String,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait HttpFetcher {
    fn new(config: HttpFetcherConfig) -> Self;

    async fn fetch(&self, url: &str) -> Result<Vec<u8>, Error>;
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait TileFetcher {
    fn new(config: HttpFetcherConfig) -> Self;

    async fn fetch_tile(&self, coords: &TileCoords) -> Result<Vec<u8>, Error>;
    fn sync_fetch_tile(&self, coords: &TileCoords) -> Result<Vec<u8>, Error>;
}
