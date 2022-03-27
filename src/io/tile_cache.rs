use crate::coords::{Quadkey, WorldTileCoords};
use crate::io::LayerResult;
use std::collections::{btree_map, BTreeMap, HashSet};

#[derive(Default)]
pub struct TileCache {
    index: BTreeMap<Quadkey, Vec<LayerResult>>,
}

impl TileCache {
    pub fn new() -> Self {
        Self {
            index: BTreeMap::new(),
        }
    }

    pub fn push(&mut self, result: LayerResult) {
        if let Some(entry) = result
            .get_coords()
            .build_quad_key()
            .map(|key| self.index.entry(key))
        {
            match entry {
                btree_map::Entry::Vacant(entry) => {
                    entry.insert(vec![result]);
                }
                btree_map::Entry::Occupied(mut entry) => {
                    entry.get_mut().push(result);
                }
            }
        }
    }

    pub fn get_tessellated_layers_at(
        &self,
        coords: &WorldTileCoords,
        skip_layers: &HashSet<&str>,
    ) -> Vec<&LayerResult> {
        let mut ret = Vec::with_capacity(10);

        if let Some(results) = coords.build_quad_key().and_then(|key| self.index.get(&key)) {
            for result in results {
                if !skip_layers.contains(&result.layer_name()) {
                    ret.push(result);
                }
            }
        }

        ret
    }

    pub fn retain_missing_layer_names(
        &self,
        coords: &WorldTileCoords,
        layers: &mut HashSet<String>,
    ) {
        if let Some(results) = coords.build_quad_key().and_then(|key| self.index.get(&key)) {
            let tessellated_set: HashSet<String> = results
                .iter()
                .map(|tessellated_layer| tessellated_layer.layer_name().to_string())
                .collect();

            layers.retain(|layer| !tessellated_set.contains(layer));
        }
    }

    pub fn is_layers_missing(&self, coords: &WorldTileCoords, layers: &HashSet<String>) -> bool {
        if let Some(results) = coords.build_quad_key().and_then(|key| self.index.get(&key)) {
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
