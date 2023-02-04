use crate::{
    coords::{LatLon, WorldCoords, Zoom},
    ecs::{
        component::EntityComponent,
        resource::{Resource, Resources},
    },
    io::{geometry_index::GeometryIndex, tile_repository::TileRepository},
    view_state::ViewState,
    window::WindowSize,
};

pub struct Entity {
    id: u64,
}

pub struct EntityMut<'w> {
    world: &'w mut World,
    entity: Entity,
}

impl<'w> EntityMut<'w> {
    pub fn insert<T: EntityComponent>(&mut self, component: T) -> &mut Self {
        unimplemented!()
    }
}

pub struct World {
    pub resources: Resources,
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

    pub fn spawn(&mut self) -> EntityMut {
        EntityMut {
            world: self,
            entity: Entity { id: 0 },
        }
    }
}
