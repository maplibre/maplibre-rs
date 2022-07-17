//! Handles IO related processing as well as multithreading.

use std::{collections::HashSet, fmt};

use crate::coords::WorldTileCoords;

pub mod scheduler;
pub mod source_client;
#[cfg(feature = "embed-static-tiles")]
pub mod static_tile_fetcher;
pub mod tile_request_state;

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
