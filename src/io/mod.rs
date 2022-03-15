//! Handles IO related processing as well as multithreading.

use crate::coords::WorldTileCoords;

use crate::render::ShaderVertex;
use crate::tessellation::{IndexDataType, OverAlignedVertexBuffer};

use std::collections::HashSet;
use std::fmt;
use vector_tile::tile::Layer;

pub mod scheduler;
pub mod static_tile_fetcher;
pub mod tile_cache;

#[derive(Clone)]
pub enum TileFetchResult {
    Unavailable {
        coords: WorldTileCoords,
    },
    Tile {
        coords: WorldTileCoords,
        data: Box<[u8]>,
    },
}

#[derive(Clone)]
pub enum TileTessellateResult {
    Tile { request_id: TileRequestID },
    Layer(LayerResult),
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
