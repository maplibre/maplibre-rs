use std::ops::{Deref, DerefMut};

use cgmath::Angle;

use crate::{
    coords::{ViewRegion, WorldCoords, Zoom, ZoomLevel, TILE_SIZE},
    render::camera::{Camera, Perspective, ViewProjection},
    util::ChangeObserver,
    window::WindowSize,
};

const VIEW_REGION_PADDING: i32 = 1;
const MAX_N_TILES: usize = 128;

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
            (position.x, position.y, 0.0),
            cgmath::Deg(-90.0),
            pitch.into(),
            window_size.width(),
            window_size.height(),
        );

        let perspective = Perspective::new(
            window_size.width(),
            window_size.height(),
            cgmath::Rad(0.6435011087932844),
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

    pub fn create_view_region(&self, visible_level: ZoomLevel) -> Option<ViewRegion> {
        self.camera
            .view_region_bounding_box(&self.view_projection().invert())
            .map(|bounding_box| {
                ViewRegion::new(
                    bounding_box,
                    VIEW_REGION_PADDING,
                    MAX_N_TILES,
                    *self.zoom,
                    visible_level,
                )
            })
    }

    pub fn view_projection(&self) -> ViewProjection {
        self.camera.calc_view_proj(&self.perspective)
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
