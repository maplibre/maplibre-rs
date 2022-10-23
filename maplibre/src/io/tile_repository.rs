//! Tile cache.

use std::collections::{btree_map, BTreeMap, HashSet};

use geozero::mvt::tile;

use crate::{
    coords::{Quadkey, WorldTileCoords},
    render::ShaderVertex,
    tessellation::{IndexDataType, OverAlignedVertexBuffer},
};

/// A layer which is stored for future use.
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

#[derive(Eq, PartialEq)]
pub enum TileStatus {
    Pending,
    Failed,
    Success,
}

/// Stores multiple [StoredLayers](StoredLayer).
pub struct StoredTile {
    layers: Vec<StoredLayer>,
    status: TileStatus,
}

impl StoredTile {
    pub fn new() -> Self {
        Self {
            layers: vec![],
            status: TileStatus::Pending,
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
                    panic!("Can not add a tessellated layer if no request has been started before.")
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

    /// Checks if a layer has been fetched.
    pub fn needs_fetching(&self, coords: &WorldTileCoords) -> bool {
        if let Some(_) = coords.build_quad_key().and_then(|key| self.tree.get(&key)) {
            return false;
        }
        true
    }

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
        }
    }

    /// Removes all the cached tessellate layers that are not contained within the given
    /// layers hashset.
    pub fn retain_missing_layer_names(
        &self,
        coords: &WorldTileCoords,
        layers: &mut HashSet<String>,
    ) {
        if let Some(cached_tile) = coords.build_quad_key().and_then(|key| self.tree.get(&key)) {
            let tessellated_set: HashSet<String> = cached_tile
                .layers
                .iter()
                .map(|tessellated_layer| tessellated_layer.layer_name().to_string())
                .collect();

            layers.retain(|layer| !tessellated_set.contains(layer));
        }
    }

    /// Checks if a layer is missing from the given layers set at the given coords.
    pub fn is_layers_missing(&self, coords: &WorldTileCoords, layers: &HashSet<String>) -> bool {
        if let Some(cached_tile) = coords.build_quad_key().and_then(|key| self.tree.get(&key)) {
            let tessellated_set: HashSet<&str> = cached_tile
                .layers
                .iter()
                .map(|tessellated_layer| tessellated_layer.layer_name())
                .collect();

            for layer in layers {
                if !tessellated_set.contains(layer.as_str()) {
                    return true;
                }
            }

            return false;
        }
        true
    }
}
