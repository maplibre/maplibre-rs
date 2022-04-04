//! Handles IO related processing as well as multithreading.

use crate::coords::WorldTileCoords;

use crate::render::ShaderVertex;
use crate::tessellation::{IndexDataType, OverAlignedVertexBuffer};

use crate::io::geometry_index::TileIndex;
use std::collections::HashSet;
use std::fmt;

use vector_tile::tile::Layer;

mod geometry_index;
pub mod scheduler;
pub mod static_tile_fetcher;
pub mod tile_cache;

pub enum TileFetchResult {
    Unavailable {
        coords: WorldTileCoords,
    },
    Tile {
        coords: WorldTileCoords,
        data: Box<[u8]>,
    },
}

impl fmt::Debug for TileFetchResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TileFetchResult({})",
            match self {
                TileFetchResult::Unavailable { coords, .. } => coords,
                TileFetchResult::Tile { coords, .. } => coords,
            }
        )
    }
}

pub struct TileIndexResult {
    request_id: TileRequestID,
    coords: WorldTileCoords,
    index: TileIndex,
}

pub enum TileTessellateResult {
    Tile { request_id: TileRequestID },
    Layer(LayerTessellateResult),
}

pub enum LayerTessellateResult {
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

impl fmt::Debug for LayerTessellateResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LayerResult{}", self.get_coords())
    }
}

impl LayerTessellateResult {
    pub fn get_coords(&self) -> WorldTileCoords {
        match self {
            LayerTessellateResult::UnavailableLayer { coords, .. } => *coords,
            LayerTessellateResult::TessellatedLayer { coords, .. } => *coords,
        }
    }

    pub fn layer_name(&self) -> &str {
        match self {
            LayerTessellateResult::UnavailableLayer { layer_name, .. } => layer_name.as_str(),
            LayerTessellateResult::TessellatedLayer { layer_data, .. } => layer_data.name(),
        }
    }
}

#[derive(Clone)]
pub struct TileRequest {
    pub coords: WorldTileCoords,
    pub layers: HashSet<String>,
}

impl fmt::Debug for TileRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TileRequest({})", &self.coords)
    }
}

pub type TileRequestID = u32;
