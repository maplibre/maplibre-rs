use crate::coords::WorldTileCoords;
use crate::io::LayerResult;
use std::collections::{btree_map, BTreeMap, HashSet};

pub struct TileCache {
    index: BTreeMap<WorldTileCoords, Vec<LayerResult>>,
}

impl TileCache {
    pub fn new() -> Self {
        Self {
            index: BTreeMap::new(),
        }
    }

    pub fn push(&mut self, result: LayerResult) {
        match self.index.entry(result.get_coords()) {
            btree_map::Entry::Vacant(entry) => {
                entry.insert(vec![result]);
            }
            btree_map::Entry::Occupied(mut entry) => {
                entry.get_mut().push(result);
            }
        }
    }

    pub fn get_tessellated_layers_at(
        &self,
        coords: &WorldTileCoords,
        skip_layers: &HashSet<String>,
    ) -> Vec<LayerResult> {
        let mut ret = Vec::new();

        if let Some(results) = self.index.get(coords) {
            for result in results {
                if !skip_layers.contains(&result.layer_name().to_string()) {
                    ret.push(result.clone());
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
        if let Some(results) = self.index.get(coords) {
            let tessellated_set: HashSet<String> = results
                .iter()
                .map(|tessellated_layer| tessellated_layer.layer_name().to_string())
                .collect();

            layers.retain(|layer| !tessellated_set.contains(layer));
        }
    }
}
