use crate::coords::{LatLon, WorldCoords, Zoom, ZoomLevel, TILE_SIZE};
use crate::render::camera::{Camera, Perspective, ViewProjection};
use crate::tile::tile_repository::TileRepository;
use crate::util::ChangeObserver;
use crate::{Renderer, Style, WindowSize};

/// Stores the camera configuration.
pub struct ViewState {
    pub zoom: ChangeObserver<Zoom>,
    pub camera: ChangeObserver<Camera>,
    pub perspective: Perspective,
}

impl ViewState {
    pub fn new(window_size: &WindowSize, zoom: Zoom, center: LatLon, pitch: f64) -> Self {
        let position = WorldCoords::from_lat_lon(center, zoom);
        let camera = Camera::new(
            (position.x, position.y, 150.0),
            cgmath::Deg(-90.0),
            cgmath::Deg(pitch),
            window_size.width(),
            window_size.height(),
        );

        let perspective = Perspective::new(
            window_size.width(),
            window_size.height(),
            cgmath::Deg(110.0),
            // in tile.vertex.wgsl we are setting each layer's final `z` in ndc space to `z_index`.
            // This means that regardless of the `znear` value all layers will be rendered as part
            // of the near plane.
            // These values have been selected experimentally:
            // https://www.sjbaker.org/steve/omniv/love_your_z_buffer.html
            1024.0,
            2048.0,
        );

        Self {
            zoom: ChangeObserver::new(zoom),
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
