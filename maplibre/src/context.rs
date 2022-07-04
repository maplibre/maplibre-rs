use crate::coords::{Zoom, ZoomLevel, TILE_SIZE};
use crate::io::tile_repository::TileRepository;
use crate::render::camera::{Camera, Perspective, ViewProjection};
use crate::util::ChangeObserver;
use crate::{Renderer, Style, WindowSize};
use cgmath::Angle;
use std::ops::Div;

/// Stores the camera configuration.
pub struct ViewState {
    pub zoom: ChangeObserver<Zoom>,
    pub camera: ChangeObserver<Camera>,
    pub perspective: Perspective,
}

impl ViewState {
    pub fn new<P: Into<cgmath::Rad<f64>>>(window_size: &WindowSize, fovy: P) -> Self {
        let center = TILE_SIZE / 2.0;
        let fovy = fovy.into();
        let height = center / (fovy / 2.0).tan();
        let camera = Camera::new(
            (center, center, height),
            cgmath::Deg(-90.0),
            cgmath::Deg(0.0),
            window_size.width(),
            window_size.height(),
        );

        let perspective = Perspective::new(
            window_size.width(),
            window_size.height(),
            fovy,
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

    pub fn visible_level(&self) -> ZoomLevel {
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

/// Stores the context of the map.
pub struct MapContext {
    pub view_state: ViewState,
    pub style: Style,

    pub tile_repository: TileRepository,
    pub renderer: Renderer,
}
