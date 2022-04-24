use crate::coords::{Quadkey, WorldTileCoords};

use crate::io::LayerTessellateMessage;

use std::collections::{btree_map, BTreeMap, HashSet};

pub struct CachedTile {
    layers: Vec<LayerTessellateMessage>,
}

impl CachedTile {
    pub fn new(first_layer: LayerTessellateMessage) -> Self {
        Self {
            layers: vec![first_layer],
        }
    }
}

#[derive(Default)]
pub struct TileCache {
    cache: BTreeMap<Quadkey, CachedTile>,
}

impl TileCache {
    pub fn new() -> Self {
        Self {
            cache: BTreeMap::new(),
        }
    }

    pub fn put_tessellated_layer(&mut self, message: LayerTessellateMessage) {
        if let Some(entry) = message
            .get_coords()
            .build_quad_key()
            .map(|key| self.cache.entry(key))
        {
            match entry {
                btree_map::Entry::Vacant(entry) => {
                    entry.insert(CachedTile::new(message));
                }
                btree_map::Entry::Occupied(mut entry) => {
                    entry.get_mut().layers.push(message);
                }
            }
        }
    }

    pub fn iter_tessellated_layers_at(
        &self,
        coords: &WorldTileCoords,
    ) -> Option<impl Iterator<Item = &LayerTessellateMessage> + '_> {
        coords
            .build_quad_key()
            .and_then(|key| self.cache.get(&key))
            .map(|results| results.layers.iter())
    }

    pub fn retain_missing_layer_names(
        &self,
        coords: &WorldTileCoords,
        layers: &mut HashSet<String>,
    ) {
        if let Some(cached_tile) = coords.build_quad_key().and_then(|key| self.cache.get(&key)) {
            let tessellated_set: HashSet<String> = cached_tile
                .layers
                .iter()
                .map(|tessellated_layer| tessellated_layer.layer_name().to_string())
                .collect();

            layers.retain(|layer| !tessellated_set.contains(layer));
        }
    }

    pub fn is_layers_missing(&self, coords: &WorldTileCoords, layers: &HashSet<String>) -> bool {
        if let Some(cached_tile) = coords.build_quad_key().and_then(|key| self.cache.get(&key)) {
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
