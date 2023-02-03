use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use downcast_rs::Downcast;

use crate::{
    coords::{LatLon, WorldCoords, Zoom},
    ecs::component::Component,
    io::{geometry_index::GeometryIndex, tile_repository::TileRepository},
    view_state::ViewState,
    window::WindowSize,
};

#[derive(Default)]
struct Resources {
    resources: Vec<Box<dyn Resource>>,
    index: HashMap<TypeId, usize>,
}

pub trait Resource: Send + Sync + 'static {}

impl<T> Resource for T where T: Send + Sync + 'static {}

pub struct Entity {
    id: u64,
}

pub struct EntityMut<'w> {
    world: &'w mut World,
    entity: Entity,
}

impl<'w> EntityMut<'w> {
    pub fn insert<T: Component>(&mut self, value: T) -> &mut Self {
        unimplemented!()
    }
}

pub struct World {
    resources: Resources,
    pub view_state: ViewState,
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
            resources: Resources::default(),
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

    pub fn insert_resource<R: Resource>(&mut self, value: R) {
        self.resources.resources.push(Box::new(value))
    }

    pub fn remove_resource<R: Resource>(&mut self) {
        if let Some(index) = self.resources.index.get(&TypeId::of::<R>()) {
            self.resources.resources.swap_remove(*index);
            let moved = &self.resources.resources[*index];
            self.resources.index.insert(moved.type_id(), *index);
        }
    }

    pub fn get_resource<R: Resource>(&self) -> Option<&R> {
        if let Some(index) = self.resources.index.get(&TypeId::of::<R>()) {
            return Some(
                self.resources.resources[*index]
                    .as_any()
                    .downcast_ref()
                    .unwrap(),
            );
        }
        return None;
    }

    /// Gets a mutable reference to the resource of the given type if it exists
    #[inline]
    pub fn get_resource_mut<R: Resource>(&mut self) -> Option<&mut R> {
        if let Some(index) = self.resources.index.get(&TypeId::of::<R>()) {
            return Some(
                self.resources.resources[*index]
                    .as_any_mut()
                    .downcast_mut()
                    .unwrap(),
            );
        }
        return None;
    }

    pub fn spawn(&mut self) -> EntityMut {
        EntityMut {
            world: self,
            entity: Entity { id: 0 },
        }
    }
}
