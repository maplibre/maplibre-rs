use std::{
    any,
    any::TypeId,
    cell::UnsafeCell,
    collections::{btree_map, BTreeMap, HashSet},
};

use downcast_rs::{impl_downcast, Downcast};

use crate::{
    coords::{Quadkey, WorldTileCoords},
    io::geometry_index::GeometryIndex,
};

#[derive(Copy, Clone, Debug)]
pub struct Tile {
    pub coords: WorldTileCoords,
}

/// A component is data associated with an [`Entity`](crate::tcs::entity::Entity). Each entity can have
/// multiple different types of components, but only one of them per type.
pub trait TileComponent: Downcast + 'static {}
impl_downcast!(TileComponent);

#[derive(Default)]
pub struct Tiles {
    pub tiles: BTreeMap<Quadkey, Tile>,
    pub components: BTreeMap<Quadkey, Vec<UnsafeCell<Box<dyn TileComponent>>>>,
    pub geometry_index: GeometryIndex,
}

impl Tiles {
    pub fn query<Q: ComponentQuery>(&self, coords: WorldTileCoords) -> Option<Q::Item<'_>> {
        let mut global_state = GlobalQueryState::default();
        let state = <Q::State<'_> as QueryState>::create(&mut global_state);
        Q::query(self, Tile { coords }, state)
    }

    pub fn query_mut<Q: ComponentQueryMut>(
        &mut self,
        coords: WorldTileCoords,
    ) -> Option<Q::MutItem<'_>> {
        let mut global_state = GlobalQueryState::default();
        let state = <Q::State<'_> as QueryState>::create(&mut global_state);
        Q::query_mut(self, Tile { coords }, state)
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
                let tile = *tile;
                Some(TileSpawnResult { tiles: self, tile })
            } else {
                let tile = Tile { coords };
                self.tiles.insert(key, tile);
                self.components.insert(key, Vec::new());
                Some(TileSpawnResult { tiles: self, tile })
            }
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.tiles.clear();
        self.components.clear();
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
                    panic!("Can not add a component at {coords}. Entity does not exist.",)
                }
                btree_map::Entry::Occupied(mut entry) => {
                    entry.get_mut().push(UnsafeCell::new(Box::new(component)));
                }
            }
        }
        self
    }
}

#[derive(Default)]
pub struct GlobalQueryState {
    mutably_borrowed: HashSet<TypeId>,
}

pub trait QueryState<'s> {
    fn create(state: &'s mut GlobalQueryState) -> Self;
    fn clone_to<'a, S: QueryState<'a>>(&'a mut self) -> S;
}

pub struct EphemeralQueryState<'s> {
    state: &'s mut GlobalQueryState,
}

impl<'s> QueryState<'s> for EphemeralQueryState<'s> {
    fn create(state: &'s mut GlobalQueryState) -> Self {
        Self { state }
    }

    fn clone_to<'a, S: QueryState<'a>>(&'a mut self) -> S {
        S::create(self.state)
    }
}

// ComponentQuery

pub trait ComponentQuery {
    type Item<'t>;

    type State<'s>: QueryState<'s>;

    fn query<'t, 's>(
        tiles: &'t Tiles,
        tile: Tile,
        state: Self::State<'s>,
    ) -> Option<Self::Item<'t>>;
}

impl<'a, T: TileComponent> ComponentQuery for &'a T {
    type Item<'t> = &'t T;
    type State<'s> = EphemeralQueryState<'s>;

    fn query<'t, 's>(
        tiles: &'t Tiles,
        tile: Tile,
        _state: Self::State<'s>,
    ) -> Option<Self::Item<'t>> {
        let components = tiles.components.get(&tile.coords.build_quad_key()?)?;

        components
            .iter()
            // FIXME tcs: Is this safe? We cast directly to & instead of &mut
            .find(|component| unsafe {
                component.get().as_ref().unwrap().as_ref().type_id() == TypeId::of::<T>()
            })
            .map(|component| unsafe {
                component
                    .get()
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .downcast_ref()
                    .expect("inserted component has wrong TypeId")
            })
    }
}

// ComponentQueryMut

pub trait ComponentQueryMut {
    type MutItem<'t>;

    type State<'s>: QueryState<'s>;

    fn query_mut<'t, 's>(
        tiles: &'t mut Tiles,
        tile: Tile,
        state: Self::State<'s>,
    ) -> Option<Self::MutItem<'t>>;
}

impl<'a, T: TileComponent> ComponentQueryMut for &'a T {
    type MutItem<'t> = &'t T;
    type State<'s> = EphemeralQueryState<'s>;

    fn query_mut<'t, 's>(
        tiles: &'t mut Tiles,
        tile: Tile,
        state: Self::State<'s>,
    ) -> Option<Self::MutItem<'t>> {
        <&T as ComponentQuery>::query(tiles, tile, state)
    }
}

impl<'a, T: TileComponent> ComponentQueryMut for &'a mut T {
    type MutItem<'t> = &'t mut T;
    type State<'s> = EphemeralQueryState<'s>;

    fn query_mut<'t, 's>(
        tiles: &'t mut Tiles,
        tile: Tile,
        _state: Self::State<'s>,
    ) -> Option<Self::MutItem<'t>> {
        let components = tiles.components.get_mut(&tile.coords.build_quad_key()?)?;

        components
            .iter_mut()
            .find(|component| unsafe {
                component.get().as_ref().unwrap().as_ref().type_id() == TypeId::of::<T>()
            })
            .map(|component| {
                component
                    .get_mut()
                    .as_mut()
                    .downcast_mut()
                    .expect("inserted component has wrong TypeId")
            })
    }
}

// ComponentQueryUnsafe

pub trait ComponentQueryUnsafe: ComponentQueryMut {
    unsafe fn query_unsafe<'t, 's>(
        tiles: &'t Tiles,
        tile: Tile,
        state: Self::State<'s>,
    ) -> Option<Self::MutItem<'t>>;
}

impl<'a, T: TileComponent> ComponentQueryUnsafe for &'a T {
    unsafe fn query_unsafe<'t, 's>(
        tiles: &'t Tiles,
        tile: Tile,
        state: Self::State<'s>,
    ) -> Option<Self::MutItem<'t>> {
        <&T as ComponentQuery>::query(tiles, tile, state)
    }
}

impl<'a, T: TileComponent> ComponentQueryUnsafe for &'a mut T {
    /// SAFETY: Safe if tiles is borrowed mutably.
    // FIXME tcs: check if really safe
    unsafe fn query_unsafe<'t, 's>(
        tiles: &'t Tiles,
        tile: Tile,
        state: Self::State<'s>,
    ) -> Option<Self::MutItem<'t>> {
        let id = TypeId::of::<T>();
        let borrowed = &mut state.state.mutably_borrowed;

        if borrowed.contains(&id) {
            panic!(
                "tried to borrow an {} more than once mutably",
                any::type_name::<T>()
            )
        }

        borrowed.insert(id);

        let components = tiles.components.get(&tile.coords.build_quad_key()?)?;

        components
            .iter()
            .find(|component| {
                component.get().as_ref().unwrap().as_ref().type_id() == TypeId::of::<T>()
            })
            .map(|component| {
                component
                    .get()
                    .as_mut()
                    .unwrap()
                    .downcast_mut()
                    .expect("inserted component has wrong TypeId")
            })
    }
}

// Lift to tuples

impl<CQ1: ComponentQuery, CQ2: ComponentQuery> ComponentQuery for (CQ1, CQ2) {
    type Item<'t> = (CQ1::Item<'t>, CQ2::Item<'t>);
    type State<'s> = EphemeralQueryState<'s>;

    fn query<'t, 's>(
        tiles: &'t Tiles,
        tile: Tile,
        mut state: Self::State<'s>,
    ) -> Option<Self::Item<'t>> {
        Some((
            CQ1::query(tiles, tile, state.clone_to::<CQ1::State<'_>>())?,
            CQ2::query(tiles, tile, state.clone_to::<CQ2::State<'_>>())?,
        ))
    }
}

impl<
        CQ1: ComponentQueryMut + ComponentQueryUnsafe + 'static,
        CQ2: ComponentQueryMut + ComponentQueryUnsafe + 'static,
    > ComponentQueryMut for (CQ1, CQ2)
{
    type MutItem<'t> = (CQ1::MutItem<'t>, CQ2::MutItem<'t>);
    type State<'s> = EphemeralQueryState<'s>;

    fn query_mut<'t, 's>(
        tiles: &'t mut Tiles,
        tile: Tile,
        mut state: Self::State<'s>,
    ) -> Option<Self::MutItem<'t>> {
        unsafe {
            Some((
                <CQ1 as ComponentQueryUnsafe>::query_unsafe(
                    tiles,
                    tile,
                    state.clone_to::<CQ1::State<'_>>(),
                )?,
                <CQ2 as ComponentQueryUnsafe>::query_unsafe(
                    tiles,
                    tile,
                    state.clone_to::<CQ2::State<'_>>(),
                )?,
            ))
        }
    }
}
