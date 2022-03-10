use crate::coords::{TileCoords, WorldTileCoords};
use crate::io::workflow::LayerResult;
use std::collections::{btree_map, BTreeMap, HashSet};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct TileCache {
    store: Arc<Mutex<BTreeMap<WorldTileCoords, Vec<LayerResult>>>>,
}

impl TileCache {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    pub fn push(&self, result: LayerResult) -> bool {
        if let Ok(mut map) = self.store.lock() {
            match map.entry(result.get_coords()) {
                btree_map::Entry::Vacant(entry) => {
                    entry.insert(vec![result]);
                }
                btree_map::Entry::Occupied(mut entry) => {
                    entry.get_mut().push(result);
                }
            }
            true
        } else {
            false
        }
    }

    pub fn get_tessellated_layers_at(
        &self,
        coords: &WorldTileCoords,
        skip_layers: &HashSet<String>,
    ) -> Vec<LayerResult> {
        let mut ret = Vec::new();
        if let Ok(map) = self.store.try_lock() {
            if let Some(results) = map.get(coords) {
                for result in results {
                    if !skip_layers.contains(&result.layer_name().to_string()) {
                        ret.push(result.clone());
                    }
                }
            }
        }

        ret
    }

    pub fn get_missing_tessellated_layer_names_at(
        &self,
        coords: &WorldTileCoords,
        mut layers: HashSet<String>,
    ) -> Option<HashSet<String>> {
        if let Ok(loaded) = self.store.try_lock() {
            if let Some(tessellated_layers) = loaded.get(coords) {
                let tessellated_set: HashSet<String> = tessellated_layers
                    .iter()
                    .map(|tessellated_layer| tessellated_layer.layer_name().to_string())
                    .collect();

                layers.retain(|layer| !tessellated_set.contains(layer));

                Some(layers)
            } else {
                Some(layers)
            }
        } else {
            None
        }
    }
}
