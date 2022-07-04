use crate::coords::{ViewRegion, Zoom, ZoomLevel, TILE_SIZE};
use crate::io::tile_repository::TileRepository;
use crate::render::camera::{Camera, Perspective, ViewProjection};
use crate::util::ChangeObserver;
use crate::{Renderer, Style, WindowSize};

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

    pub fn create_view_region(&self) -> Option<ViewRegion> {
        self.camera
            .view_region_bounding_box(&self.view_projection().invert())
            .map(|bounding_box| {
                ViewRegion::new(bounding_box, 0, 32, *self.zoom, self.visible_level())
            })
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
