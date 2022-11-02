//! Tile cache.

use std::collections::{btree_map, BTreeMap};

use crate::{
    coords::{Quadkey, WorldTileCoords},
    render::ShaderVertex,
    tessellation::{IndexDataType, OverAlignedVertexBuffer},
};

/// A layer which is stored for future use.
#[derive(Clone)]
pub enum StoredLayer {
    UnavailableLayer {
        coords: WorldTileCoords,
        layer_name: String,
    },
    TessellatedLayer {
        coords: WorldTileCoords,
        layer_name: String,
        buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
        /// Holds for each feature the count of indices.
        feature_indices: Vec<u32>,
    },
}

impl StoredLayer {
    pub fn get_coords(&self) -> WorldTileCoords {
        match self {
            StoredLayer::UnavailableLayer { coords, .. } => *coords,
            StoredLayer::TessellatedLayer { coords, .. } => *coords,
        }
    }

    pub fn layer_name(&self) -> &str {
        match self {
            StoredLayer::UnavailableLayer { layer_name, .. } => layer_name.as_str(),
            StoredLayer::TessellatedLayer { layer_name, .. } => layer_name.as_str(),
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum TileStatus {
    Pending,
    Failed,
    Success,
}

/// Stores multiple [StoredLayers](StoredLayer).
#[derive(Clone)]
pub struct StoredTile {
    coords: WorldTileCoords,
    layers: Vec<StoredLayer>,
    status: TileStatus,
}

impl StoredTile {
    pub fn pending(coords: WorldTileCoords) -> Self {
        Self {
            coords,
            layers: vec![],
            status: TileStatus::Pending,
        }
    }

    pub fn success(coords: WorldTileCoords, layers: Vec<StoredLayer>) -> Self {
        Self {
            coords,
            layers,
            status: TileStatus::Success,
        }
    }

    pub fn failed(coords: WorldTileCoords) -> Self {
        Self {
            coords,
            layers: vec![],
            status: TileStatus::Failed,
        }
    }
}

/// Stores and provides access to a quad tree of cached tiles with world tile coords.
#[derive(Default)]
pub struct TileRepository {
    tree: BTreeMap<Quadkey, StoredTile>,
}

impl TileRepository {
    pub fn new() -> Self {
        Self {
            tree: BTreeMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.tree.clear();
    }

    /// Inserts a tessellated layer into the quad tree at its world tile coords.
    /// If the space is vacant, the tessellated layer is inserted into a new
    /// [crate::io::tile_repository::StoredLayer].
    /// If the space is occupied, the tessellated layer is added to the current
    /// [crate::io::tile_repository::StoredLayer].
    pub fn put_layer(&mut self, layer: StoredLayer) {
        if let Some(entry) = layer
            .get_coords()
            .build_quad_key()
            .map(|key| self.tree.entry(key))
        {
            match entry {
                btree_map::Entry::Vacant(_entry) => {
                    panic!("Can not add a tessellated layer if no request has been started before.")
                }
                btree_map::Entry::Occupied(mut entry) => {
                    entry.get_mut().layers.push(layer);
                }
            }
        }
    }

    pub fn put_tile(&mut self, tile: StoredTile) {
        if let Some(key) = tile.coords.build_quad_key() {
            self.tree.insert(key, tile);
        }
    }

    /// Returns the list of tessellated layers at the given world tile coords. None if tile is
    /// missing from the cache.
    pub fn iter_layers_at(
        &self,
        coords: &WorldTileCoords,
    ) -> Option<impl Iterator<Item = &StoredLayer> + '_> {
        coords
            .build_quad_key()
            .and_then(|key| self.tree.get(&key))
            .map(|results| results.layers.iter())
    }

    /// Create a new tile.
    pub fn create_tile(&mut self, coords: WorldTileCoords) -> bool {
        if let Some(entry) = coords.build_quad_key().map(|key| self.tree.entry(key)) {
            match entry {
                btree_map::Entry::Vacant(entry) => {
                    entry.insert(StoredTile::pending(coords));
                }
                _ => {}
            }
        }
        true
    }

    /// Checks if a layer has been fetched.
    pub fn has_tile(&self, coords: &WorldTileCoords) -> bool {
        if coords
            .build_quad_key()
            .and_then(|key| self.tree.get(&key))
            .is_some()
        {
            return false;
        }
        true
    }

    pub fn mark_tile_succeeded(&mut self, coords: &WorldTileCoords) {
        if let Some(cached_tile) = coords
            .build_quad_key()
            .and_then(|key| self.tree.get_mut(&key))
        {
            cached_tile.status = TileStatus::Success;
        }
    }

    /// Checks if a layer has been fetched.
    pub fn mark_tile_failed(&mut self, coords: &WorldTileCoords) {
        if let Some(cached_tile) = coords
            .build_quad_key()
            .and_then(|key| self.tree.get_mut(&key))
        {
            cached_tile.status = TileStatus::Failed;
        }
    }
}
