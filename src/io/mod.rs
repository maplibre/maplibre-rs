//! Handles IO related processing as well as multithreading.

use crate::coords::{TileCoords, WorldTileCoords};
use crate::error::Error;
use crate::render::ShaderVertex;
use crate::tessellation::{IndexDataType, OverAlignedVertexBuffer};
use async_trait::async_trait;
use std::collections::HashSet;
use std::fmt;
use vector_tile::tile::Layer;

pub mod scheduler;
pub mod static_tile_fetcher;
pub mod tile_cache;
pub mod web_tile_fetcher;

pub struct HttpFetcherConfig {
    /// Under which path should we cache requests.
    pub cache_path: String,
}

impl Default for HttpFetcherConfig {
    fn default() -> Self {
        Self {
            cache_path: ".".to_string(),
        }
    }
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

#[derive(Clone)]
pub enum TileResult {
    Unavailable {
        coords: WorldTileCoords,
    },
    Tile {
        coords: WorldTileCoords,
        data: Box<[u8]>,
    },
}

#[derive(Clone)]
pub enum LayerResult {
    UnavailableLayer {
        coords: WorldTileCoords,
        layer_name: String,
    },
    TessellatedLayer {
        coords: WorldTileCoords,
        buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
        /// Holds for each feature the count of indices
        feature_indices: Vec<u32>,
        layer_data: Layer,
    },
}

impl fmt::Debug for LayerResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LayerResult{}", self.get_coords())
    }
}

impl LayerResult {
    pub fn get_coords(&self) -> WorldTileCoords {
        match self {
            LayerResult::UnavailableLayer { coords, .. } => *coords,
            LayerResult::TessellatedLayer { coords, .. } => *coords,
        }
    }

    pub fn layer_name(&self) -> &str {
        match self {
            LayerResult::UnavailableLayer { layer_name, .. } => layer_name.as_str(),
            LayerResult::TessellatedLayer { layer_data, .. } => layer_data.name(),
        }
    }
}

#[derive(Clone)]
pub struct TileRequest {
    pub coords: WorldTileCoords,
    pub layers: HashSet<String>,
}

pub type TileRequestID = u32;
