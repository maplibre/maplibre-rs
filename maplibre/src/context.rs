use std::ops::Div;

use cgmath::Angle;

use crate::{
    coords::{LatLon, ViewRegion, WorldCoords, Zoom, ZoomLevel, TILE_SIZE},
    io::tile_repository::TileRepository,
    render::camera::{Camera, Perspective, ViewProjection},
    util::ChangeObserver,
    Renderer, Style, WindowSize,
};

/// Stores the camera configuration.
pub struct ViewState {
    pub zoom: ChangeObserver<Zoom>,
    pub camera: ChangeObserver<Camera>,
    pub perspective: Perspective,
}

impl ViewState {
    pub fn new<P: Into<cgmath::Rad<f64>>>(
        window_size: &WindowSize,
        position: WorldCoords,
        zoom: Zoom,
        pitch: f64,
        fovy: P,
    ) -> Self {
        let tile_center = TILE_SIZE / 2.0;
        let fovy = fovy.into();
        let height = tile_center / (fovy / 2.0).tan();

        let camera = Camera::new(
            (position.x, position.y, height),
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
