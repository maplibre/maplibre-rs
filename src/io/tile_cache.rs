use crate::coords::{Quadkey, WorldTileCoords};

use crate::io::LayerTessellateMessage;

use std::collections::{btree_map, BTreeMap, HashSet};

#[derive(Default)]
pub struct TileCache {
    cache: BTreeMap<Quadkey, Vec<LayerTessellateMessage>>,
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
                    entry.insert(vec![message]);
                }
                btree_map::Entry::Occupied(mut entry) => {
                    entry.get_mut().push(message);
                }
            }
        }
    }

    pub fn has_tile(&self, coords: &WorldTileCoords) -> bool {
        coords
            .build_quad_key()
            .and_then(|key| {
                self.cache.get(&key).and_then(|entries| {
                    if entries.is_empty() {
                        None
                    } else if entries.iter().all(|entry| match entry {
                        LayerTessellateMessage::UnavailableLayer { .. } => true,
                        LayerTessellateMessage::TessellatedLayer { .. } => false,
                    }) {
                        None
                    } else {
                        Some(entries)
                    }
                })
            })
            .is_some()
    }

    pub fn get_tile_coords_fallback(&self, coords: &WorldTileCoords) -> Option<WorldTileCoords> {
        let mut current = *coords;
        loop {
            if self.has_tile(&current) {
                return Some(current);
            } else if let Some(parent) = current.get_parent() {
                current = parent
            } else {
                return None;
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
            .map(|results| results.iter())
    }

    pub fn retain_missing_layer_names(
        &self,
        coords: &WorldTileCoords,
        layers: &mut HashSet<String>,
    ) {
        if let Some(results) = coords.build_quad_key().and_then(|key| self.cache.get(&key)) {
            let tessellated_set: HashSet<String> = results
                .iter()
                .map(|tessellated_layer| tessellated_layer.layer_name().to_string())
                .collect();

            layers.retain(|layer| !tessellated_set.contains(layer));
        }
    }

    pub fn is_layers_missing(&self, coords: &WorldTileCoords, layers: &HashSet<String>) -> bool {
        if let Some(results) = coords.build_quad_key().and_then(|key| self.cache.get(&key)) {
            let tessellated_set: HashSet<&str> = results
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
