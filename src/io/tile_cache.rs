use crate::coords::{InnerCoords, Quadkey, WorldCoords, WorldTileCoords, EXTENT, TILE_SIZE};
use crate::io::geometry_index::IndexGeometry;
use crate::io::{LayerTessellateResult, TileIndexResult};
use cgmath::num_traits::Pow;
use std::collections::{btree_map, BTreeMap, HashSet};

#[derive(Default)]
pub struct TileCache {
    cache_index: BTreeMap<Quadkey, Vec<LayerTessellateResult>>,
    tile_geometry_index: BTreeMap<Quadkey, TileIndexResult>,
}

impl TileCache {
    pub fn new() -> Self {
        Self {
            cache_index: BTreeMap::new(),
            tile_geometry_index: BTreeMap::new(),
        }
    }

    pub fn put_tessellation_result(&mut self, result: LayerTessellateResult) {
        if let Some(entry) = result
            .get_coords()
            .build_quad_key()
            .map(|key| self.cache_index.entry(key))
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

    pub fn put_index_result(&mut self, result: TileIndexResult) {
        result
            .coords
            .build_quad_key()
            .and_then(|key| self.tile_geometry_index.insert(key, result));
    }

    pub fn query_point(
        &self,
        world_coords: &WorldCoords,
        z: u8,
        zoom: f64,
    ) -> Option<Vec<&IndexGeometry<f64>>> {
        let world_tile_coords = world_coords.into_world_tile(z, zoom);

        if let Some(index) = world_tile_coords
            .build_quad_key()
            .and_then(|key| self.tile_geometry_index.get(&key))
        {
            let scale = 2.0.pow(z as f64 - zoom); // TODO deduplicate

            let delta_x = world_coords.x / TILE_SIZE * scale - world_tile_coords.x as f64;
            let delta_y = world_coords.y / TILE_SIZE * scale - world_tile_coords.y as f64;

            let x = delta_x * EXTENT;
            let y = delta_y * EXTENT;
            Some(index.index.point_query(InnerCoords { x, y }))
        } else {
            None
        }
    }

    pub fn has_tile(&self, coords: &WorldTileCoords) -> bool {
        coords
            .build_quad_key()
            .and_then(|key| {
                self.cache_index.get(&key).and_then(|entries| {
                    if entries.is_empty() {
                        None
                    } else if entries.iter().all(|entry| match entry {
                        LayerTessellateResult::UnavailableLayer { .. } => true,
                        LayerTessellateResult::TessellatedLayer { .. } => false,
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
    ) -> Option<impl Iterator<Item = &LayerTessellateResult> + '_> {
        coords
            .build_quad_key()
            .and_then(|key| self.cache_index.get(&key))
            .map(|results| results.iter())
    }

    pub fn retain_missing_layer_names(
        &self,
        coords: &WorldTileCoords,
        layers: &mut HashSet<String>,
    ) {
        if let Some(results) = coords
            .build_quad_key()
            .and_then(|key| self.cache_index.get(&key))
        {
            let tessellated_set: HashSet<String> = results
                .iter()
                .map(|tessellated_layer| tessellated_layer.layer_name().to_string())
                .collect();

            layers.retain(|layer| !tessellated_set.contains(layer));
        }
    }

    pub fn is_layers_missing(&self, coords: &WorldTileCoords, layers: &HashSet<String>) -> bool {
        if let Some(results) = coords
            .build_quad_key()
            .and_then(|key| self.cache_index.get(&key))
        {
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
