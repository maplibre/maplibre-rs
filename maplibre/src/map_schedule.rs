//! Stores the state of the map such as `[crate::coords::Zoom]`, `[crate::camera::Camera]`, `[crate::style::Style]`, `[crate::io::tile_cache::TileCache]` and more.

use crate::context::{MapContext, ViewState};
use crate::error::Error;
use crate::io::geometry_index::GeometryIndex;
use crate::io::scheduler::Scheduler;
use crate::io::shared_thread_state::SharedThreadState;
use crate::io::source_client::{HttpClient, HttpSourceClient, SourceClient};
use crate::io::tile_cache::TileCache;
use crate::io::tile_request_state::TileRequestState;
use crate::io::TessellateMessage;
use crate::render::register_render_stages;
use crate::schedule::{Schedule, Stage};
use crate::stages::register_stages;
use crate::style::Style;
use crate::{
    HeadedMapWindow, MapWindow, MapWindowConfig, Renderer, RendererSettings, ScheduleMethod,
    WgpuSettings, WindowSize,
};
use std::marker::PhantomData;
use std::mem;
use std::sync::{mpsc, Arc, Mutex};

pub struct PrematureMapContext {
    view_state: ViewState,
    style: Style,

    tile_cache: TileCache,
    scheduler: Box<dyn ScheduleMethod>,

    message_receiver: mpsc::Receiver<TessellateMessage>,
    shared_thread_state: SharedThreadState,

    wgpu_settings: WgpuSettings,
    renderer_settings: RendererSettings,
}

pub enum EventuallyMapContext {
    Full(MapContext),
    Premature(PrematureMapContext),
    Empty,
}

impl EventuallyMapContext {
    pub fn make_full(&mut self, renderer: Renderer) {
        let context = mem::replace(self, EventuallyMapContext::Empty);

        match context {
            EventuallyMapContext::Full(_) => {}
            EventuallyMapContext::Premature(PrematureMapContext {
                view_state,
                style,
                tile_cache,
                scheduler,
                message_receiver,
                shared_thread_state,
                wgpu_settings,
                renderer_settings,
            }) => {
                mem::replace(
                    self,
                    EventuallyMapContext::Full(MapContext {
                        view_state,
                        style,
                        tile_cache,
                        renderer,
                        scheduler,
                        message_receiver,
                        shared_thread_state,
                    }),
                );
            }
            EventuallyMapContext::Empty => {}
        }
    }
}

/// Stores the state of the map, dispatches tile fetching and caching, tessellation and drawing.
pub struct MapSchedule<MWC, SM, HC>
where
    MWC: MapWindowConfig,
    SM: ScheduleMethod,
    HC: HttpClient,
{
    map_window_config: MWC,

    map_context: EventuallyMapContext,

    schedule: Schedule,

    phantom_sm: PhantomData<SM>,
    phantom_hc: PhantomData<HC>,

    suspended: bool,
}

impl<MWC, SM, HC> MapSchedule<MWC, SM, HC>
where
    MWC: MapWindowConfig,
    SM: ScheduleMethod,
    HC: HttpClient,
{
    pub fn new(
        map_window_config: MWC,
        window_size: WindowSize,
        renderer: Option<Renderer>,
        scheduler: Scheduler<SM>,
        http_client: HC,
        style: Style,
        wgpu_settings: WgpuSettings,
        renderer_settings: RendererSettings,
    ) -> Self {
        let view_state = ViewState::new(&window_size);
        let tile_cache = TileCache::new();

        let mut schedule = Schedule::default();
        let client: SourceClient<HC> = SourceClient::Http(HttpSourceClient::new(http_client));
        register_stages(&mut schedule, client);
        register_render_stages(&mut schedule);

        let (message_sender, message_receiver) = mpsc::channel();

        let scheduler = Box::new(scheduler.take());
        let shared_thread_state = SharedThreadState {
            tile_request_state: Arc::new(Mutex::new(TileRequestState::new())),
            message_sender,
            geometry_index: Arc::new(Mutex::new(GeometryIndex::new())),
        };
        Self {
            map_window_config,
            map_context: match renderer {
                None => EventuallyMapContext::Premature(PrematureMapContext {
                    view_state,
                    style,
                    tile_cache,
                    scheduler,
                    shared_thread_state,
                    wgpu_settings,
                    message_receiver,
                    renderer_settings,
                }),
                Some(renderer) => EventuallyMapContext::Full(MapContext {
                    view_state,
                    style,
                    tile_cache,
                    renderer,
                    scheduler,
                    shared_thread_state,
                    message_receiver,
                }),
            },
            schedule,
            phantom_sm: Default::default(),
            phantom_hc: Default::default(),
            suspended: false,
        }
    }

    #[tracing::instrument(name = "update_and_redraw", skip_all)]
    pub fn update_and_redraw(&mut self) -> Result<(), Error> {
        if self.suspended {
            return Ok(());
        }

        if let EventuallyMapContext::Full(map_context) = &mut self.map_context {
            self.schedule.run(map_context)
        }

        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let EventuallyMapContext::Full(map_context) = &mut self.map_context {
            let view_state = &mut map_context.view_state;
            view_state.perspective.resize(width, height);
            view_state.camera.resize(width, height);

            map_context.renderer.resize(width, height)
        }
    }

    pub fn suspend(&mut self) {
        self.suspended = true;
    }

    pub fn resume(&mut self, window: &MWC::MapWindow)
    where
        <MWC as MapWindowConfig>::MapWindow: HeadedMapWindow,
    {
        if let EventuallyMapContext::Full(map_context) = &mut self.map_context {
            let mut renderer = &mut map_context.renderer;
            renderer
                .state
                .surface
                .recreate::<MWC>(window, &renderer.instance);
            self.suspended = false;
        }
    }

    pub fn is_initialized(&self) -> bool {
        match &self.map_context {
            EventuallyMapContext::Full(_) => true,
            _ => false,
        }
    }

    pub async fn late_init(&mut self) -> bool
    where
        <MWC as MapWindowConfig>::MapWindow: HeadedMapWindow,
    {
        match &self.map_context {
            EventuallyMapContext::Full(_) => false,
            EventuallyMapContext::Premature(PrematureMapContext {
                wgpu_settings,
                renderer_settings,
                ..
            }) => {
                let window = self.map_window_config.create();
                let renderer = Renderer::initialize::<MWC>(
                    &window,
                    wgpu_settings.clone(),
                    renderer_settings.clone(),
                )
                .await
                .unwrap();
                &self.map_context.make_full(renderer);
                true
            }
            EventuallyMapContext::Empty => false,
        }
    }

    pub fn view_state_mut(&mut self) -> &mut ViewState {
        match &mut self.map_context {
            EventuallyMapContext::Full(MapContext { view_state, .. }) => view_state,
            EventuallyMapContext::Premature(PrematureMapContext { view_state, .. }) => view_state,
            _ => panic!("should not happen"),
        }
    }
}
