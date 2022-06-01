//! Handles IO related processing as well as multithreading.

use crate::coords::WorldTileCoords;
use crate::render::ShaderVertex;
use crate::tessellation::{IndexDataType, OverAlignedVertexBuffer};
use std::collections::HashSet;
use std::fmt;
pub mod scheduler;
pub mod source_client;
pub mod static_tile_fetcher;

pub mod geometry_index;
pub mod pipeline;
pub mod pipeline_steps;
pub mod tile_repository;
pub mod tile_request_state;

pub use geozero::mvt::tile::Layer as RawLayer;

/// A request for a tile at the given coordinates and in the given layers.
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

/// The ID format for a tile request.
pub type TileRequestID = u32;
