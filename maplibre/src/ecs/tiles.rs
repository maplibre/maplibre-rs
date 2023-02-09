use std::{
    any::TypeId,
    collections::{btree_map, BTreeMap},
};

use downcast_rs::{impl_downcast, Downcast};

use crate::coords::{Quadkey, WorldTileCoords};

#[derive(Copy, Clone, Debug)]
pub struct Tile {
    pub coords: WorldTileCoords,
}

/// A component is data associated with an [`Entity`](crate::ecs::entity::Entity). Each entity can have
/// multiple different types of components, but only one of them per type.
pub trait TileComponent: Downcast + 'static {}
impl_downcast!(TileComponent);

#[derive(Default)]
pub struct Tiles {
    pub tiles: BTreeMap<Quadkey, Tile>,
    pub components: BTreeMap<Quadkey, Vec<Box<dyn TileComponent>>>,
}

impl Tiles {
    pub fn query<'t, Q: ComponentQuery>(&'t self, coords: WorldTileCoords) -> Option<Q::Item<'t>> {
        Some(Q::query(&self, Tile { coords }))
    }

    pub fn query_mut<'t, Q: ComponentQueryMut>(
        &'t mut self,
        coords: WorldTileCoords,
    ) -> Option<Q::ItemMut<'t>> {
        Some(Q::query_mut(self, Tile { coords }))
    }

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

// ComponentQuery

pub trait ComponentQuery {
    type Item<'t>;

    fn query<'t>(tiles: &'t Tiles, tile: Tile) -> Self::Item<'t>;
}

impl<'a, T: TileComponent> ComponentQuery for &'a T {
    type Item<'t> = &'t T;

    fn query<'t>(tiles: &'t Tiles, tile: Tile) -> Self::Item<'t> {
        let components = tiles
            .components
            .get(&tile.coords.build_quad_key().unwrap()) // FIXME tcs: Unwrap
            .unwrap(); // FIXME tcs: Unwrap
        components
            .iter()
            .find(|component| component.as_ref().type_id() == TypeId::of::<T>())
            .and_then(|component| component.as_ref().downcast_ref())
            .unwrap() // FIXME tcs: Unwrap
    }
}

// ComponentQueryMut

pub trait ComponentQueryMut {
    type ItemMut<'t>;
    fn query_mut<'t>(tiles: &'t mut Tiles, tile: Tile) -> Self::ItemMut<'t>;
}

impl<'a, T: TileComponent> ComponentQueryMut for &'a T {
    type ItemMut<'t> = &'t T;

    fn query_mut<'t>(tiles: &'t mut Tiles, tile: Tile) -> Self::ItemMut<'t> {
        <&T as ComponentQuery>::query(tiles, tile)
    }
}

impl<'a, T: TileComponent> ComponentQueryMut for &'a mut T {
    type ItemMut<'t> = &'t mut T;

    fn query_mut<'t>(tiles: &'t mut Tiles, tile: Tile) -> Self::ItemMut<'t> {
        let components = tiles
            .components
            .get_mut(&tile.coords.build_quad_key().unwrap()) // FIXME tcs: Unwrap
            .unwrap(); // FIXME tcs: Unwrap

        components
            .iter_mut()
            .find(|component| component.as_ref().type_id() == TypeId::of::<T>())
            .and_then(|component| component.as_mut().downcast_mut())
            .unwrap() // FIXME tcs: Unwrap
    }
}

// ComponentQueryUnsafe

pub trait ComponentQueryUnsafe: ComponentQueryMut {
    unsafe fn query_unsafe<'t>(tiles: &'t Tiles, tile: Tile) -> Self::ItemMut<'t>;
}

impl<'a, T: TileComponent> ComponentQueryUnsafe for &'a T {
    unsafe fn query_unsafe<'t>(tiles: &'t Tiles, tile: Tile) -> Self::ItemMut<'t> {
        <&T as ComponentQuery>::query(tiles, tile)
    }
}

impl<'a, T: TileComponent> ComponentQueryUnsafe for &'a mut T {
    unsafe fn query_unsafe<'t>(tiles: &'t Tiles, tile: Tile) -> Self::ItemMut<'t> {
        &mut *(<&T as ComponentQuery>::query(tiles, tile) as *const T as *mut T)
    }
}

// Lift to tuples

impl<CQ1: ComponentQuery, CQ2: ComponentQuery> ComponentQuery for (CQ1, CQ2) {
    type Item<'t> = (CQ1::Item<'t>, CQ2::Item<'t>);

    fn query<'t>(tiles: &'t Tiles, tile: Tile) -> Self::Item<'t> {
        (CQ1::query(tiles, tile), CQ2::query(tiles, tile))
    }
}

impl<
        CQ1: ComponentQueryMut + ComponentQueryUnsafe + 'static,
        CQ2: ComponentQueryMut + ComponentQueryUnsafe + 'static,
    > ComponentQueryMut for (CQ1, CQ2)
{
    type ItemMut<'t> = (CQ1::ItemMut<'t>, CQ2::ItemMut<'t>);

    fn query_mut<'t>(tiles: &'t mut Tiles, tile: Tile) -> Self::ItemMut<'t> {
        let id = TypeId::of::<Self::ItemMut<'t>>();

        unsafe {
            (
                <CQ1 as ComponentQueryUnsafe>::query_unsafe(tiles, tile),
                <CQ2 as ComponentQueryUnsafe>::query_unsafe(tiles, tile),
            )
        }
    }
}
