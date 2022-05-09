use crate::coords::{ViewRegion, WorldTileCoords, Zoom, TILE_SIZE};
use crate::error::Error;
use crate::io::geometry_index::GeometryIndex;
use crate::io::scheduler::Scheduler;
use crate::io::shared_thread_state::SharedThreadState;
use crate::io::source_client::{HTTPClient, HttpSourceClient, SourceClient};
use crate::io::tile_cache::TileCache;
use crate::io::tile_request_state::TileRequestState;
use crate::io::{TessellateMessage, TileRequest, TileTessellateMessage};
use crate::render::camera;
use crate::render::camera::{Camera, Perspective, ViewProjection};
use crate::render::render_state::RenderState;
use crate::style::Style;
use crate::util::ChangeObserver;
use crate::{MapWindow, MapWindowConfig, ScheduleMethod, WindowSize};
use std::collections::HashSet;

use std::sync::{mpsc, Arc, Mutex};

pub struct ViewState {
    zoom: ChangeObserver<Zoom>,
    pub camera: ChangeObserver<Camera>,
    pub perspective: Perspective,
}

impl ViewState {
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

pub struct MapState<MWC, SM, HC>
where
    MWC: MapWindowConfig,
    SM: ScheduleMethod,
    HC: HTTPClient,
{
    map_window_config: MWC,

    view_state: ViewState,

    render_state: Option<RenderState>,
    scheduler: Scheduler<SM>,
    message_receiver: mpsc::Receiver<TessellateMessage>,
    shared_thread_state: SharedThreadState,
    tile_cache: TileCache,

    source_client: SourceClient<HC>,

    style: Style,

    try_failed: bool,
}

impl<MWC, SM, HC> MapState<MWC, SM, HC>
where
    MWC: MapWindowConfig,
    SM: ScheduleMethod,
    HC: HTTPClient,
{
    pub fn new(
        map_window_config: MWC,
        window_size: WindowSize,
        render_state: Option<RenderState>,
        scheduler: Scheduler<SM>,
        http_client: HC,
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
            map_window_config,
            view_state: ViewState {
                zoom: ChangeObserver::default(),
                camera: ChangeObserver::new(camera),
                perspective,
            },

            render_state,
            scheduler,

            tile_cache: TileCache::new(),
            message_receiver,
            shared_thread_state: SharedThreadState {
                tile_request_state: Arc::new(Mutex::new(TileRequestState::new())),
                message_sender,
                geometry_index: Arc::new(Mutex::new(GeometryIndex::new())),
            },

            style,

            try_failed: false,
            source_client: SourceClient::Http(HttpSourceClient::new(http_client)),
        }
    }

    pub fn update_and_redraw(&mut self) -> Result<(), Error> {
        // Get data from other threads
        self.try_populate_cache();

        // Update buffers
        self.prepare_render();

        // Render buffers
        self.render_state_mut().render()?;

        #[cfg(all(feature = "enable-tracing", not(target_arch = "wasm32")))]
        tracy_client::finish_continuous_frame!();

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    fn try_populate_cache(&mut self) {
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
    fn prepare_render(&mut self) {
        let render_setup_span = tracing::span!(tracing::Level::TRACE, "setup view region");
        let _guard = render_setup_span.enter();

        let visible_level = self.view_state.visible_level();

        let view_proj = self.view_state.view_projection();

        let view_region = self
            .view_state
            .camera
            .view_region_bounding_box(&view_proj.invert())
            .map(|bounding_box| {
                ViewRegion::new(bounding_box, 0, *self.view_state.zoom, visible_level)
            });

        drop(_guard);

        if let Some(view_region) = &view_region {
            self.render_state
                .as_mut()
                .expect("render state not yet initialized. Call reinitialize().")
                .upload_tile_geometry(view_region, &self.style, &self.tile_cache);

            let zoom = self.view_state.zoom();
            self.render_state_mut()
                .update_tile_view_pattern(view_region, &view_proj, zoom);

            self.render_state_mut().update_metadata();
        }

        // TODO: Could we draw inspiration from StagingBelt (https://docs.rs/wgpu/latest/wgpu/util/struct.StagingBelt.html)?
        // TODO: What is StagingBelt for?

        if self.view_state.camera.did_change(0.05)
            || self.view_state.zoom.did_change(0.05)
            || self.try_failed
        {
            if let Some(view_region) = &view_region {
                // FIXME: We also need to request tiles from layers above if we are over the maximum zoom level
                self.try_failed = self.request_tiles_in_view(view_region);
            }

            self.render_state()
                .update_globals(&view_proj, &self.view_state.camera);
        }

        self.view_state.camera.update_reference();
        self.view_state.zoom.update_reference();
    }

    fn try_request_tile(
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

                let client = self.source_client.clone();
                let coords = *coords;

                self.scheduler
                    .schedule_method()
                    .schedule(
                        self.shared_thread_state.clone(),
                        move |state: SharedThreadState| async move {
                            match client.fetch(&coords).await {
                                Ok(data) => state
                                    .process_tile(request_id, data.into_boxed_slice())
                                    .unwrap(),
                                Err(e) => {
                                    log::error!("{:?}", &e);
                                    state.tile_unavailable(&coords, request_id).unwrap()
                                }
                            }
                        },
                    )
                    .unwrap();
            }

            Ok(false)
        } else {
            Ok(true)
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.view_state.perspective.resize(width, height);
        self.view_state.camera.resize(width, height);

        self.render_state_mut().resize(width, height)
    }

    pub fn scheduler(&self) -> &Scheduler<SM> {
        &self.scheduler
    }

    pub fn suspend(&mut self) {
        self.render_state_mut().suspend();
    }

    pub fn resume(&mut self) {
        self.render_state_mut().resume();
    }

    pub fn render_state(&self) -> &RenderState {
        self.render_state
            .as_ref()
            .expect("render state not yet initialized. Call reinitialize().")
    }

    pub fn render_state_mut(&mut self) -> &'_ mut RenderState {
        self.render_state.as_mut().unwrap()
    }

    pub fn view_state(&self) -> &ViewState {
        &self.view_state
    }

    pub fn view_state_mut(&mut self) -> &mut ViewState {
        &mut self.view_state
    }

    pub fn recreate_surface(&mut self, window: &MWC::MapWindow) {
        self.render_state
            .as_mut()
            .expect("render state not yet initialized. Call reinitialize().")
            .recreate_surface(window);
    }

    pub fn is_initialized(&self) -> bool {
        self.render_state.is_some()
    }

    pub async fn reinitialize(&mut self) {
        if self.render_state.is_none() {
            let instance = wgpu::Instance::new(wgpu::Backends::all());
            //let instance = wgpu::Instance::new(wgpu::Backends::GL);
            //let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);

            let window = MWC::MapWindow::create(&self.map_window_config);
            let window_size = window.size();

            let surface = unsafe { instance.create_surface(window.inner()) };
            let surface_config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: crate::platform::COLOR_TEXTURE_FORMAT,
                width: window_size.width(),
                height: window_size.height(),
                // present_mode: wgpu::PresentMode::Mailbox,
                present_mode: wgpu::PresentMode::Fifo, // VSync
            };
            let _window_size = window.size();
            let render_state = RenderState::initialize(instance, surface, surface_config)
                .await
                .unwrap();
            self.render_state = Some(render_state)
        }
    }
}
