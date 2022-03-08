use crate::coords::TileCoords;
use crate::io::workflow::LayerResult;
use std::collections::{btree_map, BTreeMap};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct TileCache {
    store: Arc<Mutex<BTreeMap<TileCoords, Vec<LayerResult>>>>,
}

impl TileCache {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    pub(crate) fn push(&self, result: LayerResult) -> bool {
        if let Ok(mut map) = self.store.lock() {
            match map.entry(result.get_tile_coords()) {
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

    pub(crate) fn get_tessellated_layers_at(
        &self,
        coords: &TileCoords,
        skip_layers: &Vec<String>,
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

    pub(crate) fn get_missing_tessellated_layer_names_at(
        &self,
        coords: &TileCoords,
        layers: &Vec<String>,
    ) -> Vec<String> {
        if let Ok(loaded) = self.store.try_lock() {
            if let Some(tessellated_layers) = loaded.get(coords) {
                let mut result = Vec::new();
                for layer in layers {
                    if tessellated_layers
                        .iter()
                        .find(|tessellated_layer| tessellated_layer.layer_name() == layer)
                        .is_none()
                    {
                        result.push(layer.clone());
                    }
                }
                result
            } else {
                layers.clone()
            }
        } else {
            Vec::new()
        }
    }
}
