use crate::coords::{ViewRegion, Zoom, TILE_SIZE};
use crate::input::{InputController, UpdateState};
use crate::io::scheduler::Scheduler;
use crate::io::LayerTessellateMessage;
use crate::render::camera;
use crate::render::camera::{Camera, Perspective, ViewProjection};
use crate::render::render_state::RenderState;
use crate::util::ChangeObserver;
use crate::{SurfaceFactory, WindowSize};
use std::collections::HashSet;
use std::iter;
use std::time::Duration;
use style_spec::Style;
use wgpu::SurfaceError;

pub trait Runnable<E> {
    fn run(self, event_loop: E, max_frames: Option<u64>);
}

pub struct MapState<W> {
    render_state: RenderState,

    window: W,

    zoom: ChangeObserver<Zoom>,

    scheduler: Scheduler,

    try_failed: bool,

    style: Style,

    camera: ChangeObserver<camera::Camera>,
    perspective: camera::Perspective,
}

impl<W> MapState<W> {
    pub fn new(
        window: W,
        window_size: WindowSize,
        render_state: RenderState,
        scheduler: Scheduler,
        style: Style,
    ) -> Self {
        let camera = camera::Camera::new(
            (TILE_SIZE / 2.0, TILE_SIZE / 2.0, 150.0),
            cgmath::Deg(-90.0),
            cgmath::Deg(0.0),
            window_size.width,
            window_size.height,
        );

        let perspective = camera::Perspective::new(
            window_size.width,
            window_size.height,
            cgmath::Deg(110.0),
            100.0,
            2000.0,
        );

        Self {
            render_state,
            window,
            zoom: ChangeObserver::default(),
            try_failed: false,
            style,
            scheduler,
            camera: ChangeObserver::new(camera),
            perspective,
        }
    }

    pub fn update_and_redraw(&mut self) -> Result<(), SurfaceError> {
        self.scheduler.try_populate_cache();

        self.prepare_render();
        self.render_state.render()
    }

    /// Request tiles which are currently in view
    #[tracing::instrument(skip_all)]
    fn request_tiles_in_view(&mut self, view_region: &ViewRegion) -> bool {
        let mut try_failed = false;
        let source_layers: HashSet<String> = self
            .style
            .layers
            .iter()
            .filter_map(|layer| layer.source_layer.clone())
            .collect();

        for coords in view_region.iter() {
            if coords.build_quad_key().is_some() {
                // TODO: Make tesselation depend on style?
                try_failed = self
                    .scheduler
                    .try_request_tile(&coords, &source_layers)
                    .unwrap();
            }
        }
        try_failed
    }

    #[tracing::instrument(skip_all)]
    pub fn prepare_render(&mut self) {
        let render_setup_span = tracing::span!(tracing::Level::TRACE, "setup view region");
        let _guard = render_setup_span.enter();

        let visible_level = self.visible_level();

        let view_proj = self.camera.calc_view_proj(&self.perspective);

        let view_region = self
            .camera
            .view_region_bounding_box(&view_proj.invert())
            .map(|bounding_box| ViewRegion::new(bounding_box, 0, *self.zoom, visible_level));

        drop(_guard);

        if let Some(view_region) = &view_region {
            self.render_state.upload_tile_geometry(
                view_region,
                &self.style,
                self.scheduler.get_tile_cache(),
            );

            self.render_state
                .update_tile_view_pattern(view_region, &view_proj, self.zoom());

            self.render_state.update_metadata();
        }

        // TODO: Could we draw inspiration from StagingBelt (https://docs.rs/wgpu/latest/wgpu/util/struct.StagingBelt.html)?
        // TODO: What is StagingBelt for?

        if self.camera.did_change(0.05) || self.zoom.did_change(0.05) || self.try_failed {
            if let Some(view_region) = &view_region {
                // FIXME: We also need to request tiles from layers above if we are over the maximum zoom level
                self.try_failed = self.request_tiles_in_view(view_region);
            }

            self.render_state.update_globals(&view_proj, &self.camera);
        }

        self.camera.update_reference();
        self.zoom.update_reference();
    }

    pub fn scheduler(&self) -> &Scheduler {
        &self.scheduler
    }

    pub fn window(&self) -> &W {
        &self.window
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn perspective(&self) -> &Perspective {
        &self.perspective
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width <= 0 || height <= 0 {
            return;
        }

        self.perspective.resize(width, height);
        self.camera.resize(width, height);

        self.render_state.resize(width, height)
    }

    pub fn zoom(&self) -> Zoom {
        *self.zoom
    }
    pub fn visible_level(&self) -> u8 {
        self.zoom.level()
    }

    pub fn update_zoom(&mut self, new_zoom: Zoom) {
        *self.zoom = new_zoom;
        log::info!("zoom: {}", new_zoom);
    }

    pub fn suspend(&mut self) {
        self.render_state.suspend();
    }

    pub fn resume(&mut self) {
        self.render_state.resume();
    }
}

impl<W> MapState<W>
where
    W: raw_window_handle::HasRawWindowHandle,
{
    pub fn recreate_surface(&mut self) {
        self.render_state.recreate_surface(&self.window);
    }
}
