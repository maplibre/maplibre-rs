//! Handles IO related processing as well as multithreading.

use crate::coords::WorldTileCoords;

use crate::render::ShaderVertex;
use crate::tessellation::{IndexDataType, OverAlignedVertexBuffer};

use std::collections::HashSet;
use std::fmt;

use vector_tile::tile::Layer;

mod geometry_index;
pub mod scheduler;
mod source_client;
pub mod static_tile_fetcher;
pub mod tile_cache;
mod tile_request_state;

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

pub enum TessellateMessage {
    Tile(TileTessellateMessage),
    Layer(LayerTessellateMessage),
}

pub struct TileTessellateMessage {
    request_id: TileRequestID,
}

pub enum LayerTessellateMessage {
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

impl fmt::Debug for LayerTessellateMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LayerTessellateMessage{}", self.get_coords())
    }
}

impl LayerTessellateMessage {
    pub fn get_coords(&self) -> WorldTileCoords {
        match self {
            LayerTessellateMessage::UnavailableLayer { coords, .. } => *coords,
            LayerTessellateMessage::TessellatedLayer { coords, .. } => *coords,
        }
    }

    pub fn layer_name(&self) -> &str {
        match self {
            LayerTessellateMessage::UnavailableLayer { layer_name, .. } => layer_name.as_str(),
            LayerTessellateMessage::TessellatedLayer { layer_data, .. } => layer_data.name(),
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
        write!(f, "TileRequest({}, {:?})", &self.coords, &self.layers)
    }
}

pub type TileRequestID = u32;
