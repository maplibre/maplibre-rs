//! Tile cache.

use crate::coords::{Quadkey, WorldTileCoords};
use crate::render::ShaderVertex;
use crate::tessellation::{IndexDataType, OverAlignedVertexBuffer};
use geozero::mvt::tile;
use std::collections::{btree_map, BTreeMap};

/// A layer which is stored for future use.
pub enum StoredLayer {
    UnavailableLayer {
        coords: WorldTileCoords,
        layer_name: String,
    },
    TessellatedLayer {
        coords: WorldTileCoords,
        buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
        feature_indices: Vec<u32>,
        layer_data: tile::Layer,
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
            StoredLayer::TessellatedLayer { layer_data, .. } => &layer_data.name,
        }
    }
}

#[derive(PartialEq)]
pub enum TileStatus {
    Pending,
    Failed,
    Success,
}

/// Stores multiple [StoredLayers](StoredLayer).
pub struct StoredTile {
    layers: Vec<StoredLayer>,
    status: TileStatus,
    retry: u32,
}

impl StoredTile {
    pub fn new() -> Self {
        Self {
            layers: vec![],
            status: TileStatus::Pending,
            retry: 0,
        }
    }
    pub fn new_with_first_layer(first_layer: StoredLayer) -> Self {
        Self {
            layers: vec![first_layer],
            status: TileStatus::Pending,
            retry: 0,
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

    /// Inserts a tessellated layer into the quad tree at its world tile coords.
    /// If the space is vacant, the tessellated layer is inserted into a new
    /// [crate::io::tile_repository::CachedTile].
    /// If the space is occupied, the tessellated layer is added to the current
    /// [crate::io::tile_repository::CachedTile].
    pub fn put_tessellated_layer(&mut self, layer: StoredLayer) {
        if let Some(entry) = layer
            .get_coords()
            .build_quad_key()
            .map(|key| self.tree.entry(key))
        {
            match entry {
                btree_map::Entry::Vacant(entry) => {
                    entry.insert(StoredTile::new_with_first_layer(layer));
                }
                btree_map::Entry::Occupied(mut entry) => {
                    entry.get_mut().layers.push(layer);
                }
            }
        }
    }

    /// Returns the list of tessellated layers at the given world tile coords. None if tile is
    /// missing from the cache.
    pub fn iter_tessellated_layers_at(
        &self,
        coords: &WorldTileCoords,
    ) -> Option<impl Iterator<Item = &StoredLayer> + '_> {
        coords
            .build_quad_key()
            .and_then(|key| self.tree.get(&key))
            .map(|results| results.layers.iter())
    }

    /// Create a new tile.
    pub fn create_tile(&mut self, coords: &WorldTileCoords) -> bool {
        if let Some(entry) = coords.build_quad_key().map(|key| self.tree.entry(key)) {
            match entry {
                btree_map::Entry::Vacant(entry) => {
                    entry.insert(StoredTile::new());
                }
                _ => {}
            }
        }
        true
    }

    /// Set the status to success
    pub fn success(&mut self, coords: &WorldTileCoords) {
        if let Some(cached_tile) = coords
            .build_quad_key()
            .and_then(|key| self.tree.get_mut(&key))
        {
            cached_tile.status = TileStatus::Success;
        }
    }

    /// Checks if a layer has been fetched.
    pub fn fail(&mut self, coords: &WorldTileCoords) {
        if let Some(cached_tile) = coords
            .build_quad_key()
            .and_then(|key| self.tree.get_mut(&key))
        {
            cached_tile.status = TileStatus::Failed;
            cached_tile.retry += 1;
        }
    }

    /// Checks if a layer has been fetched.
    pub fn needs_fetching(&self, coords: &WorldTileCoords) -> bool {
        if let Some(cached_tile) = coords.build_quad_key().and_then(|key| self.tree.get(&key)) {
            return cached_tile.status == TileStatus::Failed && cached_tile.retry < 3;
        }
        true
    }
}
