use crate::coords::{Zoom, TILE_SIZE};
use crate::io::tile_cache::TileCache;
use crate::render::camera::{Camera, Perspective, ViewProjection};
use crate::util::ChangeObserver;
use crate::{Renderer, ScheduleMethod, Style, WindowSize};
use std::sync::mpsc;

/// Stores the camera configuration.
pub struct ViewState {
    pub zoom: ChangeObserver<Zoom>,
    pub camera: ChangeObserver<Camera>,
    pub perspective: Perspective,
}

impl ViewState {
    pub fn new(window_size: &WindowSize) -> Self {
        let camera = Camera::new(
            (TILE_SIZE / 2.0, TILE_SIZE / 2.0, 150.0),
            cgmath::Deg(-90.0),
            cgmath::Deg(0.0),
            window_size.width(),
            window_size.height(),
        );

        let perspective = Perspective::new(
            window_size.width(),
            window_size.height(),
            cgmath::Deg(110.0),
            100.0,
            2000.0,
        );

        Self {
            zoom: ChangeObserver::default(),
            camera: ChangeObserver::new(camera),
            perspective,
        }
    }

    pub fn view_projection(&self) -> ViewProjection {
        self.camera.calc_view_proj(&self.perspective)
    }

    pub fn visible_level(&self) -> u8 {
        self.zoom.level()
    }

    pub fn zoom(&self) -> Zoom {
        *self.zoom
    }

    pub fn update_zoom(&mut self, new_zoom: Zoom) {
        *self.zoom = new_zoom;
        log::info!("zoom: {}", new_zoom);
    }
}

pub struct MapContext {
    pub view_state: ViewState,
    pub style: Style,

    pub tile_cache: TileCache,
    pub renderer: Renderer,
}
