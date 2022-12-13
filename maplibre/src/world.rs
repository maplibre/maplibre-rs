use std::ops::{Deref, DerefMut};

use cgmath::Angle;

use crate::{
    coords::{LatLon, ViewRegion, WorldCoords, Zoom, ZoomLevel, TILE_SIZE},
    io::{geometry_index::GeometryIndex, tile_repository::TileRepository},
    render::camera::{Camera, Perspective, ViewProjection},
    util::ChangeObserver,
    window::WindowSize,
};

pub struct World {
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
}

/// Stores the camera configuration.
pub struct ViewState {
    zoom: ChangeObserver<Zoom>,
    camera: ChangeObserver<Camera>,
    perspective: Perspective,
}

impl ViewState {
    pub fn new<F: Into<cgmath::Rad<f64>>, P: Into<cgmath::Deg<f64>>>(
        window_size: WindowSize,
        position: WorldCoords,
        zoom: Zoom,
        pitch: P,
        fovy: F,
    ) -> Self {
        let tile_center = TILE_SIZE / 2.0;
        let fovy = fovy.into();
        let height = tile_center / (fovy / 2.0).tan();

        let camera = Camera::new(
            (position.x, position.y, height),
            cgmath::Deg(-90.0),
            pitch.into(),
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

    pub fn resize(&mut self, width: u32, height: u32) {
        self.perspective.resize(width, height);
        self.camera.resize(width, height);
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

    pub fn did_zoom_change(&self) -> bool {
        self.zoom.did_change(0.05)
    }

    pub fn update_zoom(&mut self, new_zoom: Zoom) {
        *self.zoom = new_zoom;
        log::info!("zoom: {}", new_zoom);
    }

    pub fn camera(&self) -> &Camera {
        self.camera.deref()
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        self.camera.deref_mut()
    }

    pub fn did_camera_change(&self) -> bool {
        self.camera.did_change(0.05)
    }

    pub fn update_references(&mut self) {
        self.camera.update_reference();
        self.zoom.update_reference();
    }
}
