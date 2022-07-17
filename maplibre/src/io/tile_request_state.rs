//! Tile request state.

use std::collections::{HashMap, HashSet};

use crate::{
    coords::WorldTileCoords,
    io::{TileRequest, TileRequestID},
};

/// Stores a map of pending requests, coords and the current tile being requested.
#[derive(Default)]
pub struct TileRequestState {
    current_id: TileRequestID,
    pending_tile_requests: HashMap<TileRequestID, TileRequest>,
    pending_coords: HashSet<WorldTileCoords>,
}

impl TileRequestState {
    pub fn new() -> Self {
        Self {
            current_id: 1,
            pending_tile_requests: Default::default(),
            pending_coords: Default::default(),
        }
    }

    pub fn is_tile_request_pending(&self, coords: &WorldTileCoords) -> bool {
        self.pending_coords.contains(coords)
    }

    pub fn start_tile_request(&mut self, tile_request: TileRequest) -> Option<TileRequestID> {
        if self.is_tile_request_pending(&tile_request.coords) {
            return None;
        }

        self.pending_coords.insert(tile_request.coords);
        let id = self.current_id;
        self.pending_tile_requests.insert(id, tile_request);
        self.current_id += 1;
        Some(id)
    }

    pub fn finish_tile_request(&mut self, id: TileRequestID) -> Option<TileRequest> {
        self.pending_tile_requests.remove(&id).map(|request| {
            self.pending_coords.remove(&request.coords);
            request
        })
    }

    pub fn get_tile_request(&self, id: TileRequestID) -> Option<&TileRequest> {
        self.pending_tile_requests.get(&id)
    }
}
