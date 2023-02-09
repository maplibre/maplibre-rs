use std::default::Default;

use crate::{
    coords::{LatLon, WorldCoords, Zoom},
    ecs::{resources::Resources, tiles::Tiles},
    io::geometry_index::GeometryIndex,
    view_state::ViewState,
    window::WindowSize,
};

pub struct World {
    pub resources: Resources,
    pub tiles: Tiles,

    pub view_state: ViewState, // FIXME: create resource
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

        let geometry_index = GeometryIndex::new();

        World {
            resources: Default::default(),
            tiles: Default::default(),
            view_state,
            geometry_index,
        }
    }

    pub fn view_state(&self) -> &ViewState {
        &self.view_state
    }

    pub fn view_state_mut(&mut self) -> &mut ViewState {
        &mut self.view_state
    }
}
