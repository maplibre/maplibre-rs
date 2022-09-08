use std::{marker::PhantomData, mem};

use crate::{
    context::{MapContext, ViewState},
    coords::{LatLon, WorldCoords, Zoom, TILE_SIZE},
    error::Error,
    io::{
        scheduler::Scheduler,
        source_client::{HttpClient, HttpSourceClient},
        tile_repository::TileRepository,
    },
    render::{create_default_render_graph, register_default_render_stages},
    schedule::{Schedule, Stage},
    stages::register_stages,
    style::Style,
    HeadedMapWindow, MapWindowConfig, Renderer, RendererSettings, ScheduleMethod, WgpuSettings,
    WindowSize,
};

/// Stores the state of the map, dispatches tile fetching and caching, tessellation and drawing.
pub struct InteractiveMapSchedule<MWC, SM, HC>
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

impl<MWC, SM, HC> InteractiveMapSchedule<MWC, SM, HC>
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
        let zoom = style.zoom.map(|zoom| Zoom::new(zoom)).unwrap_or_default();
        let position = style
            .center
            .map(|center| WorldCoords::from_lat_lon(LatLon::new(center[0], center[1]), zoom))
            .unwrap_or_default();
        let pitch = style.pitch.unwrap_or_default();
        let view_state = ViewState::new(&window_size, position, zoom, pitch, cgmath::Deg(110.0));

        let tile_repository = TileRepository::new();
        let mut schedule = Schedule::default();

        let http_source_client: HttpSourceClient<HC> = HttpSourceClient::new(http_client);
        register_stages(&mut schedule, http_source_client, Box::new(scheduler));

        let graph = create_default_render_graph().unwrap();
        register_default_render_stages(graph, &mut schedule);

        Self {
            map_window_config,
            map_context: match renderer {
                None => EventuallyMapContext::Premature(PrematureMapContext {
                    view_state,
                    style,
                    tile_repository,
                    wgpu_settings,
                    renderer_settings,
                }),
                Some(renderer) => EventuallyMapContext::Full(MapContext {
                    view_state,
                    style,
                    tile_repository,
                    renderer,
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
            let renderer = &mut map_context.renderer;
            renderer
                .state
                .recreate_surface::<MWC::MapWindow>(window, &renderer.instance);
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
                let renderer =
                    Renderer::initialize(&window, wgpu_settings.clone(), renderer_settings.clone())
                        .await
                        .unwrap();
                self.map_context.make_full(renderer);
                true
            }
            EventuallyMapContext::_Uninitialized => false,
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

pub struct PrematureMapContext {
    view_state: ViewState,
    style: Style,

    tile_repository: TileRepository,

    wgpu_settings: WgpuSettings,
    renderer_settings: RendererSettings,
}

pub enum EventuallyMapContext {
    Full(MapContext),
    Premature(PrematureMapContext),
    _Uninitialized,
}

impl EventuallyMapContext {
    pub fn make_full(&mut self, renderer: Renderer) {
        let context = mem::replace(self, EventuallyMapContext::_Uninitialized);

        match context {
            EventuallyMapContext::Full(_) => {}
            EventuallyMapContext::Premature(PrematureMapContext {
                view_state,
                style,
                tile_repository,
                ..
            }) => {
                *self = EventuallyMapContext::Full(MapContext {
                    view_state,
                    style,
                    tile_repository,
                    renderer,
                });
            }
            EventuallyMapContext::_Uninitialized => {}
        }
    }
}
