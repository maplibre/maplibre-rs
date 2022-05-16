//! Stores the state of the map such as `[crate::coords::Zoom]`, `[crate::camera::Camera]`, `[crate::style::Style]`, `[crate::io::tile_cache::TileCache]` and more.

use crate::context::MapContext;
use crate::coords::{ViewRegion, WorldTileCoords, Zoom, TILE_SIZE};
use crate::error::Error;
use crate::io::geometry_index::GeometryIndex;
use crate::io::scheduler::Scheduler;
use crate::io::shared_thread_state::SharedThreadState;
use crate::io::source_client::{HTTPClient, HttpSourceClient, SourceClient};
use crate::io::tile_cache::TileCache;
use crate::io::tile_request_state::TileRequestState;
use crate::io::{TessellateMessage, TileRequest, TileTessellateMessage};
use crate::render::camera::{Camera, Perspective, ViewProjection};
use crate::render::{camera, register_render_stages};
use crate::schedule::{Schedule, Stage};
use crate::stages::register_stages;
use crate::style::Style;
use crate::util::ChangeObserver;
use crate::{MapWindow, MapWindowConfig, Renderer, ScheduleMethod, WindowSize};
use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::{mpsc, Arc, Mutex};

/// Stores the camera configuration.
pub struct ViewState {
    pub zoom: ChangeObserver<Zoom>,
    pub camera: ChangeObserver<Camera>,
    pub perspective: Perspective,
}

impl ViewState {
    pub fn new(window_size: &WindowSize) -> Self {
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

/// Stores the state of the map, dispatches tile fetching and caching, tessellation and drawing.
///
/// FIXME: MapState may not follow the Single-responsibility principle, as it not only stores
/// the state of the map but also the rendering, caching, etc.
pub struct MapState<MWC, SM, HC>
where
    MWC: MapWindowConfig,
    SM: ScheduleMethod,
    HC: HTTPClient,
{
    map_window_config: MWC,

    map_context: MapContext,

    schedule: Schedule,

    message_receiver: mpsc::Receiver<TessellateMessage>,

    phantom_sm: PhantomData<SM>,
    phantom_hc: PhantomData<HC>,
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
        renderer: Option<Renderer>,
        scheduler: Scheduler<SM>,
        http_client: HC,
        style: Style,
    ) -> Self {
        let view_state = ViewState::new(&window_size);
        let tile_cache = TileCache::new();

        let (message_sender, message_receiver) = mpsc::channel();
        let shared_thread_state = SharedThreadState {
            tile_request_state: Arc::new(Mutex::new(TileRequestState::new())),
            message_sender,
            geometry_index: Arc::new(Mutex::new(GeometryIndex::new())),
        };

        let mut schedule = Schedule::default();

        if let Some(ref renderer) = renderer {
            let client: SourceClient<HC> = SourceClient::Http(HttpSourceClient::new(http_client));
            register_stages(&mut schedule, client);
            register_render_stages(&mut schedule);
        }

        Self {
            map_window_config,
            map_context: MapContext {
                view_state,
                style,
                tile_cache,
                renderer: renderer.unwrap(),
                scheduler: Box::new(scheduler.take()),
                shared_thread_state,
            },
            schedule,
            message_receiver,
            phantom_sm: Default::default(),
            phantom_hc: Default::default(),
        }
    }

    pub fn update_and_redraw(&mut self) -> Result<(), Error> {
        // Get data from other threads
        self.try_populate_cache();

        self.schedule.run(&mut self.map_context);

        #[cfg(all(feature = "enable-tracing", not(target_arch = "wasm32")))]
        tracy_client::finish_continuous_frame!();

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    fn try_populate_cache(&mut self) {
        let tile_cache = &mut self.map_context.tile_cache;
        let shared_thread_state = &mut self.map_context.shared_thread_state;

        if let Ok(result) = self.message_receiver.try_recv() {
            match result {
                TessellateMessage::Layer(layer_result) => {
                    tracing::trace!(
                        "Layer {} at {} reached main thread",
                        layer_result.layer_name(),
                        layer_result.get_coords()
                    );
                    tile_cache.put_tessellated_layer(layer_result);
                }
                TessellateMessage::Tile(TileTessellateMessage { request_id, coords }) => loop {
                    if let Ok(mut tile_request_state) =
                        shared_thread_state.tile_request_state.try_lock()
                    {
                        tile_request_state.finish_tile_request(request_id);
                        tracing::trace!("Tile at {} finished loading", coords);
                        break;
                    }
                },
            }
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.map_context
            .view_state
            .perspective
            .resize(width, height);
        self.map_context.view_state.camera.resize(width, height);

        // FIXME self.renderer_mut().resize(width, height)
    }

    /*FIXME pub fn scheduler(&self) -> &Scheduler<SM> {
        &self.scheduler
    }*/

    pub fn suspend(&mut self) {
        // FIXME
    }

    pub fn resume(&mut self) {
        // FIXME
    }

    /*FIXME pub fn renderer(&self) -> &Renderer {
        self.renderer
            .as_ref()
            .expect("render state not yet initialized. Call reinitialize().")
    }*/

    /*FIXME pub fn renderer_mut(&mut self) -> &'_ mut Renderer {
        self.renderer.as_mut().unwrap()
    }*/

    pub fn view_state(&mut self) -> &ViewState {
        &self.map_context.view_state
    }
    pub fn view_state_mut(&mut self) -> &mut ViewState {
        &mut self.map_context.view_state
    }

    /*FIXME: pub fn recreate_surface(&mut self, window: &MWC::MapWindow) {
        self.renderer
            .as_mut()
            .expect("render state not yet initialized. Call reinitialize().")
            .recreate_surface(window);
    }*/

    /*FIXME pub fn is_initialized(&self) -> bool {
        self.renderer.is_some()
    }*/

    pub async fn reinitialize(&mut self) {
        /*FIXME if self.renderer.is_none() {
            let window = MWC::MapWindow::create(&self.map_window_config);
            let renderer = Renderer::initialize(&window).await.unwrap();
            self.renderer = Some(renderer)
        }*/
    }
}
