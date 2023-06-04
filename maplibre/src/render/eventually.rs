use std::mem;

use crate::{coords::WorldTileCoords, render::tile_view_pattern::HasTile, tcs::world::World};

/// Wrapper around a resource which can be initialized or uninitialized.
/// Uninitialized resourced can be initialized by calling [`Eventually::initialize()`].
pub enum Eventually<T> {
    Initialized(T),
    Uninitialized,
}

pub trait HasChanged {
    type Criteria: Eq;

    fn has_changed(&self, criteria: &Self::Criteria) -> bool;
}

impl<T> HasChanged for Option<T>
where
    T: HasChanged,
{
    type Criteria = T::Criteria;

    fn has_changed(&self, criteria: &Self::Criteria) -> bool {
        match self {
            None => true,
            Some(value) => value.has_changed(criteria),
        }
    }
}

impl<T> Eventually<T>
where
    T: HasChanged,
{
    #[tracing::instrument(name = "reinitialize", skip_all)]
    pub fn reinitialize(&mut self, f: impl FnOnce() -> T, criteria: &T::Criteria) {
        let should_replace = match &self {
            Eventually::Initialized(current) => current.has_changed(criteria),
            Eventually::Uninitialized => true,
        };

        if should_replace {
            *self = Eventually::Initialized(f());
        }
    }
}
impl<T> Eventually<T> {
    #[tracing::instrument(name = "initialize", skip_all)]
    pub fn initialize(&mut self, f: impl FnOnce() -> T) {
        if let Eventually::Uninitialized = self {
            *self = Eventually::Initialized(f());
        }
    }

    pub fn take(&mut self) -> Eventually<T> {
        mem::replace(self, Eventually::Uninitialized)
    }

    pub fn expect_initialized_mut(&mut self, message: &str) -> &mut T {
        match self {
            Eventually::Initialized(value) => value,
            Eventually::Uninitialized => panic!("{message}"),
        }
    }
}

impl<T> Default for Eventually<T> {
    fn default() -> Self {
        Eventually::Uninitialized
    }
}

impl<T> HasTile for Eventually<T>
where
    T: HasTile,
{
    fn has_tile(&self, coords: WorldTileCoords, world: &World) -> bool {
        match self {
            Eventually::Initialized(value) => value.has_tile(coords, world),
            Eventually::Uninitialized => false,
        }
    }
}
