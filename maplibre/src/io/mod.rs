//! Handles IO related processing as well as multithreading.

use std::{collections::HashSet, fmt};

use serde::{Deserialize, Serialize};

use crate::coords::WorldTileCoords;

pub mod apc;
pub mod geometry_index;
pub mod pipeline;
pub mod scheduler;
pub mod source_client;
#[cfg(feature = "embed-static-tiles")]
pub mod static_tile_fetcher;
pub mod tile_pipelines;
pub mod tile_repository;
pub mod transferables;

pub use geozero::mvt::tile::Layer as RawLayer;

/// A request for a tile at the given coordinates and in the given layers.
#[derive(Clone, Serialize, Deserialize)]
pub struct TileRequest {
    pub coords: WorldTileCoords,
    pub layers: HashSet<String>,
}

impl fmt::Debug for TileRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TileRequest({}, {:?})", &self.coords, &self.layers)
    }
}
