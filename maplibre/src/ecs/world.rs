use std::{
    any::{Any, TypeId},
    collections::{btree_map, BTreeMap},
    default::Default,
};

use crate::{
    coords::{LatLon, Quadkey, WorldCoords, WorldTileCoords, Zoom},
    ecs::{
        component::TileComponent,
        resource::{Resource, Resources},
    },
    io::{geometry_index::GeometryIndex, tile_repository::TileRepository},
    render::render_phase::RenderCommand,
    view_state::ViewState,
    window::WindowSize,
};

#[derive(Copy, Clone, Debug)]
pub struct Tile {
    pub coords: WorldTileCoords,
}

pub struct TileRef<'w> {
    world: &'w World,
    tile: Tile,
}

impl<'w> TileRef<'w> {
    // FIXME: Duplicate components
    pub fn query_components<T: TileComponent>(&self) -> Vec<&T> {
        if let Some(key) = self.tile.coords.build_quad_key() {
            if let Some(components) = self.world.components.get(&key) {
                components
                    .iter()
                    .filter(|component| component.as_ref().type_id() == TypeId::of::<T>())
                    .filter_map(|component| component.as_ref().downcast_ref())
                    .collect::<Vec<_>>()
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    pub fn query_component<T: TileComponent>(&self) -> Option<&T> {
        if let Some(key) = self.tile.coords.build_quad_key() {
            if let Some(components) = self.world.components.get(&key) {
                components
                    .iter()
                    .find(|component| component.as_ref().type_id() == TypeId::of::<T>())
                    .and_then(|component| component.as_ref().downcast_ref())
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub struct TileMut<'w> {
    world: &'w mut World,
    tile: Tile,
}

impl<'w> TileMut<'w> {
    pub fn insert<T: TileComponent>(&mut self, component: T) -> &mut Self {
        let components = &mut self.world.components;
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

pub struct World {
    pub resources: Resources,
    pub tiles: BTreeMap<Quadkey, Tile>,
    pub components: BTreeMap<Quadkey, Vec<Box<dyn TileComponent>>>,

    pub view_state: ViewState, // FIXME: create resource
    pub tile_repository: TileRepository,
    pub geometry_index: GeometryIndex,
}

impl World {
    pub fn new_at<P: Into<cgmath::Deg<f64>>>(
        window_size: WindowSize,
        initial_center: LatLon,
        initial_zoom: Zoom,
        pitch: P,
    ) -> Self {
        Self::new(
            window_size,
            WorldCoords::from_lat_lon(initial_center, initial_zoom),
            initial_zoom,
            pitch,
        )
    }

    pub fn new<P: Into<cgmath::Deg<f64>>>(
        window_size: WindowSize,
        initial_center: WorldCoords,
        initial_zoom: Zoom,
        pitch: P,
    ) -> Self {
        let position = initial_center;
        let view_state = ViewState::new(
            window_size,
            position,
            initial_zoom,
            pitch,
            cgmath::Deg(110.0),
        );

        let tile_repository = TileRepository::new();
        let geometry_index = GeometryIndex::new();

        World {
            resources: Default::default(),
            tiles: Default::default(),
            components: Default::default(),
            view_state,
            tile_repository,
            geometry_index,
        }
    }

    pub fn view_state(&self) -> &ViewState {
        &self.view_state
    }

    pub fn view_state_mut(&mut self) -> &mut ViewState {
        &mut self.view_state
    }

    pub fn init_resource<R: Resource + Default>(&mut self) {
        self.insert_resource(R::default());
    }

    pub fn insert_resource<R: Resource>(&mut self, resource: R) {
        self.resources.insert(resource);
    }

    pub fn remove_resource<R: Resource>(&mut self) {
        self.resources.remove::<R>()
    }

    /// Gets a reference to the resource of the given type if it exists
    pub fn get_resource<R: Resource>(&self) -> &R {
        self.resources.get::<R>().expect("Resource does not exist")
    }

    /// Gets a mutable reference to the resource of the given type if it exists
    pub fn get_resource_mut<R: Resource>(&mut self) -> &mut R {
        self.resources
            .get_mut::<R>()
            .expect("Resource does not exist")
    }

    pub fn query_tile(&self, coords: WorldTileCoords) -> Option<TileRef> {
        if let Some(key) = coords.build_quad_key() {
            Some(TileRef {
                world: self,
                tile: self.tiles.get(&key).cloned().unwrap(), // FIXME
            })
        } else {
            None
        }
    }

    pub fn query_tile_mut(&mut self, coords: WorldTileCoords) -> Option<TileMut> {
        if let Some(key) = coords.build_quad_key() {
            if let Some(tile) = self.tiles.get(&key) {
                let tile = tile.clone();
                Some(TileMut { world: self, tile })
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn spawn_mut(&mut self, coords: WorldTileCoords) -> Option<TileMut> {
        if let Some(key) = coords.build_quad_key() {
            if let Some(tile) = self.tiles.get(&key) {
                let tile = tile.clone();
                Some(TileMut { world: self, tile })
            } else {
                let tile = Tile { coords };
                self.tiles.insert(key, tile);
                self.components.insert(key, Vec::new());
                Some(TileMut {
                    world: self,
                    tile: tile.clone(),
                })
            }
        } else {
            None
        }
    }
}
