//! Tile cache.

use std::collections::{btree_map, btree_map::Entry, BTreeMap};

use bytemuck::Pod;
use thiserror::Error;

use crate::{
    coords::{Quadkey, WorldTileCoords},
    render::{
        resource::{BufferPool, Queue},
        ShaderVertex,
    },
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
#[derive(Error, Debug)]
pub enum MarkError {
    #[error("no pending tile at coords")]
    NoPendingTile,
    #[error("unable to construct quadkey")]
    QuadKey,
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
                    panic!("Can not add a tessellated layer at {} if no request has been started before. \
                    We might received a tile which was not requested.", layer.get_coords())
                }
                btree_map::Entry::Occupied(mut entry) => {
                    entry.get_mut().layers.push(layer);
                }
            }
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
            .and_then(|tile| {
                if tile.status == TileStatus::Success {
                    Some(tile)
                } else {
                    None
                }
            })
            .map(|tile| tile.layers.iter())
    }

    /// Returns the list of tessellated layers at the given world tile coords, which are loaded in
    /// the BufferPool
    pub fn iter_loaded_layers_at<Q: Queue<B>, B, V: Pod, I: Pod, TM: Pod, FM: Pod>(
        &self,
        buffer_pool: &BufferPool<Q, B, V, I, TM, FM>,
        coords: &WorldTileCoords,
    ) -> Option<Vec<&StoredLayer>> {
        let loaded_layers = buffer_pool.get_loaded_layers_at(coords).unwrap_or_default();

        self.iter_layers_at(coords).map(|layers| {
            layers
                .filter(|result| !loaded_layers.contains(&result.layer_name()))
                .collect::<Vec<_>>()
        })
    }

    /// Checks fetching of a tile has been started
    pub fn is_tile_pending_or_done(&self, coords: &WorldTileCoords) -> bool {
        if coords
            .build_quad_key()
            .and_then(|key| self.tree.get(&key))
            .is_some()
        {
            return false;
        }
        true
    }

    /// Mark the tile at `coords` pending in this tile repository.
    pub fn mark_tile_pending(&mut self, coords: WorldTileCoords) -> Result<(), MarkError> {
        let Some(key) = coords.build_quad_key() else { return Err(MarkError::QuadKey); };

        match self.tree.entry(key) {
            Entry::Vacant(entry) => {
                entry.insert(StoredTile::pending(coords));
            }
            Entry::Occupied(mut entry) => {
                entry.get_mut().status = TileStatus::Pending;
            }
        }

        Ok(())
    }

    /// Mark the tile at `coords` succeeded in this tile repository. Only succeeds if there is a
    /// pending tile at `coords`.
    pub fn mark_tile_succeeded(&mut self, coords: &WorldTileCoords) -> Result<(), MarkError> {
        self.mark_tile(coords, TileStatus::Success)
    }

    /// Mark the tile at `coords` failed in this tile repository. Only succeeds if there is a
    /// pending tile at `coords`.
    pub fn mark_tile_failed(&mut self, coords: &WorldTileCoords) -> Result<(), MarkError> {
        self.mark_tile(coords, TileStatus::Failed)
    }

    fn mark_tile(&mut self, coords: &WorldTileCoords, status: TileStatus) -> Result<(), MarkError> {
        let Some(key) = coords.build_quad_key() else { return Err(MarkError::QuadKey); };

        if let Entry::Occupied(mut entry) = self.tree.entry(key) {
            entry.get_mut().status = status;
            Ok(())
        } else {
            Err(MarkError::NoPendingTile)
        }
    }

    pub fn put_tile(&mut self, tile: StoredTile) {
        if let Some(key) = tile.coords.build_quad_key() {
            self.tree.insert(key, tile);
        }
    }

    pub fn clear(&mut self) {
        self.tree.clear();
    }
}
