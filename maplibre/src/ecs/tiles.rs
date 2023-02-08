use std::{
    any::TypeId,
    collections::{btree_map, BTreeMap},
};

use crate::{
    coords::{Quadkey, WorldTileCoords},
    ecs::{component::TileComponent, world::Tile, Mut, Ref},
};

#[derive(Default)]
pub struct Tiles {
    pub tiles: BTreeMap<Quadkey, Tile>,
    pub components: BTreeMap<Quadkey, Vec<Box<dyn TileComponent>>>,
}

impl Tiles {
    pub fn query_component<'t, Q: ComponentQuery>(
        &'t self,
        coords: WorldTileCoords,
    ) -> Option<Q::Item<'t>> {
        Some(Q::get_component(&self, Tile { coords }))
    }

    pub fn query_component_mut<'t, Q: ComponentQuery>(
        &'t mut self,
        coords: WorldTileCoords,
    ) -> Option<Q::Item<'t>> {
        Some(Q::get_component_mut(self, Tile { coords }))
    }

    /*pub fn query_tile(&self, coords: WorldTileCoords) -> Option<TileRef> {
        if let Some(key) = coords.build_quad_key() {
            Some(TileRef {
                tiles: self,
                tile: self.tiles.get(&key).cloned().unwrap(), // FIXME
            })
        } else {
            None
        }
    }

    pub fn query_tile_mut(&mut self, coords: WorldTileCoords) -> Option<TileSpawnResult> {
        if let Some(key) = coords.build_quad_key() {
            if let Some(tile) = self.tiles.get(&key) {
                let tile = tile.clone();
                Some(TileSpawnResult { tiles: self, tile })
            } else {
                None
            }
        } else {
            None
        }
    }*/

    pub fn exists(&self, coords: WorldTileCoords) -> bool {
        if let Some(key) = coords.build_quad_key() {
            self.tiles.get(&key).is_some()
        } else {
            false
        }
    }

    pub fn spawn_mut(&mut self, coords: WorldTileCoords) -> Option<TileSpawnResult> {
        if let Some(key) = coords.build_quad_key() {
            if let Some(tile) = self.tiles.get(&key) {
                let tile = tile.clone();
                Some(TileSpawnResult { tiles: self, tile })
            } else {
                let tile = Tile { coords };
                self.tiles.insert(key, tile);
                self.components.insert(key, Vec::new());
                Some(TileSpawnResult {
                    tiles: self,
                    tile: tile.clone(),
                })
            }
        } else {
            None
        }
    }
}

pub struct TileSpawnResult<'t> {
    tiles: &'t mut Tiles,
    tile: Tile,
}

impl<'w> TileSpawnResult<'w> {
    pub fn insert<T: TileComponent>(&mut self, component: T) -> &mut Self {
        let components = &mut self.tiles.components;
        let coords = self.tile.coords;

        if let Some(entry) = coords.build_quad_key().map(|key| components.entry(key)) {
            match entry {
                btree_map::Entry::Vacant(_entry) => {
                    panic!(
                        "Can not add a component at {}. Entity does not exist.",
                        coords
                    )
                }
                btree_map::Entry::Occupied(mut entry) => {
                    entry.get_mut().push(Box::new(component));
                }
            }
        }
        self
    }
}

pub trait ComponentQuery {
    type Item<'a>;

    fn get_component<'t>(tiles: &'t Tiles, tile: Tile) -> Self::Item<'t>;
    fn get_component_mut<'t>(tiles: &'t mut Tiles, tile: Tile) -> Self::Item<'t>;
}

impl<'t, T: TileComponent> ComponentQuery for Ref<'t, T> {
    type Item<'a> = Ref<'a, T>;

    fn get_component<'a>(tiles: &'a Tiles, tile: Tile) -> Self::Item<'a> {
        let components = tiles
            .components
            .get(&tile.coords.build_quad_key().unwrap())
            .unwrap();
        Ref {
            value: components
                .iter()
                .find(|component| component.as_ref().type_id() == TypeId::of::<T>())
                .and_then(|component| component.as_ref().downcast_ref())
                .unwrap(),
        }
    }

    fn get_component_mut<'a>(tiles: &'a mut Tiles, tile: Tile) -> Self::Item<'a> {
        Self::get_component(tiles, tile)
    }
}

impl<'t, T: TileComponent> ComponentQuery for Mut<'t, T> {
    type Item<'a> = Mut<'a, T>;

    fn get_component<'a>(tiles: &'a Tiles, tile: Tile) -> Self::Item<'a> {
        panic!("provide an inmutable World to query inmutable")
    }

    fn get_component_mut<'a>(tiles: &'a mut Tiles, tile: Tile) -> Self::Item<'a> {
        let components = tiles
            .components
            .get_mut(&tile.coords.build_quad_key().unwrap())
            .unwrap();
        Mut {
            value: components
                .iter_mut()
                .find(|component| component.as_ref().type_id() == TypeId::of::<T>())
                .and_then(|component| component.as_mut().downcast_mut())
                .unwrap(),
        }
    }
}

impl<CQ1: ComponentQuery> ComponentQuery for (CQ1,) {
    type Item<'a> = (CQ1::Item<'a>,);

    fn get_component<'a>(tiles: &'a Tiles, tile: Tile) -> Self::Item<'a> {
        (CQ1::get_component(tiles, tile),)
    }

    fn get_component_mut<'a>(tiles: &'a mut Tiles, tile: Tile) -> Self::Item<'a> {
        (CQ1::get_component_mut(tiles, tile),)
    }
}

impl<CQ1: ComponentQuery, CQ2: ComponentQuery> ComponentQuery for (CQ1, CQ2) {
    type Item<'a> = (CQ1::Item<'a>, CQ2::Item<'a>);

    fn get_component<'a>(tiles: &'a Tiles, tile: Tile) -> Self::Item<'a> {
        (
            CQ1::get_component(tiles, tile),
            CQ2::get_component(tiles, tile),
        )
    }

    fn get_component_mut<'a>(tiles: &'a mut Tiles, tile: Tile) -> Self::Item<'a> {
        let components = tiles
            .components
            .get_mut(&tile.coords.build_quad_key().unwrap())
            .unwrap();

        let results = components
            .iter_mut()
            .filter(|component| component.as_ref().type_id() == TypeId::of::<CQ2::Item<'a>>())
            .flat_map(|component| component.as_mut().downcast_mut::<CQ2::Item<'a>>())
            .collect::<Vec<_>>();

        (results[0], results[1])
    }
}
