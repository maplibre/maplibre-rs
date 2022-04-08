use crate::coords::{ViewRegion, WorldTileCoords, Zoom, TILE_SIZE};

use crate::io::scheduler::Scheduler;

use crate::render::camera;
use crate::render::camera::{Camera, Perspective};
use crate::render::render_state::RenderState;
use crate::util::ChangeObserver;
use crate::WindowSize;
use std::collections::HashSet;
use std::sync::{mpsc, Arc, Mutex};

use crate::error::Error;
use crate::io::geometry_index::GeometryIndex;
use crate::io::shared_thread_state::SharedThreadState;
use crate::io::source_client::{HttpSourceClient, SourceClient};
use crate::io::tile_cache::TileCache;
use crate::io::tile_request_state::TileRequestState;
use crate::io::{TessellateMessage, TileRequest, TileTessellateMessage};
use style_spec::Style;
use wgpu::SurfaceError;

pub trait Runnable<E> {
    fn run(self, event_loop: E, max_frames: Option<u64>);
}

pub type Channel<T> = (mpsc::Sender<T>, mpsc::Receiver<T>);

pub struct MapState<W> {
    window: W,

    zoom: ChangeObserver<Zoom>,
    camera: ChangeObserver<camera::Camera>,
    perspective: camera::Perspective,

    render_state: RenderState,
    scheduler: Scheduler,
    message_receiver: mpsc::Receiver<TessellateMessage>,
    shared_thread_state: SharedThreadState,
    tile_cache: TileCache,

    style: Style,

    try_failed: bool,
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
            window_size.width(),
            window_size.height(),
        );

        let perspective = camera::Perspective::new(
            window_size.width(),
            window_size.height(),
            cgmath::Deg(110.0),
            100.0,
            2000.0,
        );

        let (message_sender, message_receiver) = mpsc::channel();

        Self {
            render_state,
            window,
            zoom: ChangeObserver::default(),
            try_failed: false,
            style,
            scheduler,
            camera: ChangeObserver::new(camera),
            perspective,
            message_receiver,
            tile_cache: TileCache::new(),
            shared_thread_state: SharedThreadState {
                tile_request_state: Arc::new(Mutex::new(TileRequestState::new())),
                message_sender,
                geometry_index: Arc::new(Mutex::new(GeometryIndex::new())),
            },
        }
    }

    pub fn update_and_redraw(&mut self) -> Result<(), SurfaceError> {
        self.try_populate_cache();

        self.prepare_render();
        self.render_state.render()
    }

    #[tracing::instrument(skip_all)]
    pub fn try_populate_cache(&mut self) {
        if let Ok(result) = self.message_receiver.try_recv() {
            match result {
                TessellateMessage::Layer(layer_result) => {
                    tracing::trace!(
                        "Layer {} at {} reached main thread",
                        layer_result.layer_name(),
                        layer_result.get_coords()
                    );
                    self.tile_cache.put_tessellated_layer(layer_result);
                }
                TessellateMessage::Tile(TileTessellateMessage { request_id, coords }) => loop {
                    if let Ok(mut tile_request_state) =
                        self.shared_thread_state.tile_request_state.try_lock()
                    {
                        tile_request_state.finish_tile_request(request_id);
                        tracing::trace!("Tile at {} finished loading", coords);
                        break;
                    }
                },
            }
        }
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
                try_failed = self.try_request_tile(&coords, &source_layers).unwrap();
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
            self.render_state
                .upload_tile_geometry(view_region, &self.style, &self.tile_cache);

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

    pub fn try_request_tile(
        &mut self,
        coords: &WorldTileCoords,
        layers: &HashSet<String>,
    ) -> Result<bool, Error> {
        if !self.tile_cache.is_layers_missing(coords, layers) {
            return Ok(false);
        }

        if let Ok(mut tile_request_state) = self.shared_thread_state.tile_request_state.try_lock() {
            if let Some(request_id) = tile_request_state.start_tile_request(TileRequest {
                coords: *coords,
                layers: layers.clone(),
            }) {
                tracing::info!("new tile request: {}", &coords);

                // The following snippet can be added instead of the next code block to demonstrate
                // an understanable approach of fetching
                /*#[cfg(target_arch = "wasm32")]
                if let Some(tile_coords) = coords.into_tile(TileAddressingScheme::TMS) {
                    crate::platform::legacy_webworker_fetcher::request_tile(
                        request_id,
                        tile_coords,
                    );
                }*/

                {
                    let client = SourceClient::Http(HttpSourceClient::new());
                    let copied_coords = *coords;

                    let future_fn = move |state: SharedThreadState| async move {
                        if let Ok(data) = client.fetch(&copied_coords).await {
                            state
                                .process_tile(request_id, data.into_boxed_slice())
                                .unwrap();
                        } else {
                            state.tile_unavailable(request_id).unwrap();
                        }
                    };

                    #[cfg(target_arch = "wasm32")]
                    self.scheduler
                        .schedule_method()
                        .schedule(self.shared_thread_state.clone(), future_fn)
                        .unwrap();
                    #[cfg(not(target_arch = "wasm32"))]
                    self.scheduler
                        .schedule_method()
                        .schedule(self.shared_thread_state.clone(), future_fn)
                        .unwrap();
                }
            }

            Ok(false)
        } else {
            Ok(true)
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.perspective.resize(width, height);
        self.camera.resize(width, height);

        self.render_state.resize(width, height)
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
