use crate::io::scheduler::{ScheduleMethod, Scheduler};
use crate::io::source_client::HTTPClient;
use crate::map_state::{MapState, Runnable};
use crate::render::render_state::{MapSurface, RenderState, WindowMapSurface};
use crate::style::Style;
use crate::window::{MapWindow, WindowSize};
use std::marker::PhantomData;

pub mod coords;
pub mod error;
pub mod headless;
pub mod io;
pub mod platform;
pub mod style;
pub mod window;
pub mod winit;

// Used for benchmarking
pub mod benchmarking;

// Internal modules
pub(crate) mod input;
pub(crate) mod map_state;
pub(crate) mod render;
pub(crate) mod tessellation;
pub(crate) mod tilejson;
pub(crate) mod util;

pub struct Map<W, E, SM, HC>
where
    W: MapWindow<E>,
    SM: ScheduleMethod,
    HC: HTTPClient,
{
    map_state: MapState<W, E, SM, HC>,
    event_loop: E,
}

impl<W, E, SM, HC> Map<W, E, SM, HC>
where
    MapState<W, E, SM, HC>: Runnable<E>,
    W: MapWindow<E>,
    SM: ScheduleMethod,
    HC: HTTPClient,
{
    pub fn run(self) {
        self.run_with_optionally_max_frames(None);
    }

    pub fn run_with_max_frames(self, max_frames: u64) {
        self.run_with_optionally_max_frames(Some(max_frames));
    }

    pub fn run_with_optionally_max_frames(self, max_frames: Option<u64>) {
        self.map_state.run(self.event_loop, max_frames);
    }
}

pub struct UninitializedMap<W, E, SM, HC>
where
    SM: ScheduleMethod,
    HC: HTTPClient,
{
    scheduler: Scheduler<SM>,
    http_client: HC,

    style: Style,

    phantom_w: PhantomData<W>,
    phantom_e: PhantomData<E>,
}

impl<W, E, SM, HC> UninitializedMap<W, E, SM, HC>
where
    W: MapWindow<E>,
    SM: ScheduleMethod,
    HC: HTTPClient,
{
    pub async fn initialize(self) -> Map<W, E, SM, HC> {
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        //let instance = wgpu::Instance::new(wgpu::Backends::GL);
        //let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);

        let (window, surface, window_size, event_loop) = W::create(&instance);
        let render_state = RenderState::initialize(instance, surface).await;
        Map {
            map_state: MapState::new(
                window,
                window_size,
                render_state,
                self.scheduler,
                self.http_client,
                self.style,
            ),
            event_loop,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<W, E, SM, HC> UninitializedMap<W, E, SM, HC>
where
    W: MapWindow<E>,
    MapState<W, E, SM, HC>: Runnable<E>,
    SM: ScheduleMethod,
    HC: HTTPClient,
{
    pub fn run_sync(self) {
        self.run_sync_with_optionally_max_frames(None);
    }

    pub fn run_sync_with_max_frames(self, max_frames: u64) {
        self.run_sync_with_optionally_max_frames(Some(max_frames))
    }

    fn run_sync_with_optionally_max_frames(self, max_frames: Option<u64>) {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_io()
            .enable_time()
            .on_thread_start(|| {
                #[cfg(feature = "enable-tracing")]
                tracy_client::set_thread_name("tokio-runtime-worker");
            })
            .build()
            .unwrap()
            .block_on(async {
                self.initialize()
                    .await
                    .run_with_optionally_max_frames(max_frames);
            })
    }
}

pub struct MapBuilder<W, E, SM, HC>
where
    SM: ScheduleMethod,
{
    schedule_method: Option<SM>,
    scheduler: Option<Scheduler<SM>>,
    http_client: Option<HC>,
    style: Option<Style>,

    phantom_w: PhantomData<W>,
    phantom_e: PhantomData<E>,
}

impl<W, E, SM, HC> MapBuilder<W, E, SM, HC>
where
    MapState<W, E, SM, HC>: Runnable<E>,
    W: MapWindow<E>,
    SM: ScheduleMethod,
    HC: HTTPClient,
{
    pub fn new() -> Self {
        Self {
            schedule_method: None,
            scheduler: None,
            http_client: None,
            style: None,
            phantom_w: Default::default(),
            phantom_e: Default::default(),
        }
    }

    pub fn with_schedule_method(mut self, schedule_method: SM) -> Self {
        self.schedule_method = Some(schedule_method);
        self
    }

    pub fn with_http_client(mut self, http_client: HC) -> Self {
        self.http_client = Some(http_client);
        self
    }

    pub fn with_existing_scheduler(mut self, scheduler: Scheduler<SM>) -> Self {
        self.scheduler = Some(scheduler);
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    pub fn build(self) -> UninitializedMap<W, E, SM, HC> {
        let scheduler = self
            .scheduler
            .unwrap_or_else(|| Scheduler::new(self.schedule_method.unwrap()));
        let style = self.style.unwrap_or_default();

        UninitializedMap {
            scheduler,
            http_client: self.http_client.unwrap(),
            style,
            phantom_w: Default::default(),
            phantom_e: Default::default(),
        }
    }
}
