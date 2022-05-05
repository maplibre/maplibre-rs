use crate::io::scheduler::{ScheduleMethod, Scheduler};
use crate::io::source_client::HTTPClient;
use crate::map_state::{MapState, Runnable};
use crate::render::render_state::RenderState;
use crate::style::Style;
use crate::window::{MapWindow, WindowFactory, WindowSize};
use std::marker::PhantomData;

pub mod coords;
pub mod error;
pub mod io;
pub mod platform;
pub mod style;
pub mod window;

// Used for benchmarking
pub mod benchmarking;

// Internal modules
pub(crate) mod input;
pub(crate) mod map_state;
pub(crate) mod render;
pub(crate) mod tessellation;
pub(crate) mod tilejson;
pub(crate) mod util;
pub(crate) mod winit;

pub struct Map<W, E, SM, HC>
where
    W: MapWindow,
    SM: ScheduleMethod,
    HC: HTTPClient,
{
    map_state: MapState<W, SM, HC>,
    event_loop: E,
}

impl<W, E, SM, HC> Map<W, E, SM, HC>
where
    MapState<W, SM, HC>: Runnable<E>,
    W: MapWindow,
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
    window: W,
    event_loop: E,
    scheduler: Scheduler<SM>,
    http_client: HC,
    style: Style,
}

impl<W, E, SM, HC> UninitializedMap<W, E, SM, HC>
where
    W: MapWindow + raw_window_handle::HasRawWindowHandle,
    SM: ScheduleMethod,
    HC: HTTPClient,
{
    pub async fn initialize(self) -> Map<W, E, SM, HC> {
        #[cfg(target_os = "android")]
        // On android we can not get the dimensions of the window initially. Therefore, we use a
        // fallback until the window is ready to deliver its correct bounds.
        let window_size = self.window.size().unwrap_or_default();

        #[cfg(not(target_os = "android"))]
        let window_size = self
            .window
            .size()
            .expect("failed to get window dimensions.");

        let render_state = RenderState::initialize(&self.window, window_size).await;
        Map {
            map_state: MapState::new(
                self.window,
                window_size,
                render_state,
                self.scheduler,
                self.http_client,
                self.style,
            ),
            event_loop: self.event_loop,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<W, E, SM, HC> UninitializedMap<W, E, SM, HC>
where
    W: MapWindow + raw_window_handle::HasRawWindowHandle,
    MapState<W, SM, HC>: Runnable<E>,
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
    window_factory: Box<WindowFactory<W, E>>,
    schedule_method: Option<SM>,
    scheduler: Option<Scheduler<SM>>,
    http_client: Option<HC>,
    style: Option<Style>,
}

impl<W, E, SM, HC> MapBuilder<W, E, SM, HC>
where
    MapState<W, SM, HC>: Runnable<E>,
    W: MapWindow + raw_window_handle::HasRawWindowHandle,
    SM: ScheduleMethod,
    HC: HTTPClient,
{
    pub fn new(create_window: Box<WindowFactory<W, E>>) -> Self {
        Self {
            window_factory: create_window,
            schedule_method: None,
            scheduler: None,
            http_client: None,
            style: None,
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
        let (window, event_loop) = (self.window_factory)();

        let scheduler = self
            .scheduler
            .unwrap_or_else(|| Scheduler::new(self.schedule_method.unwrap()));
        let style = self.style.unwrap_or_default();

        UninitializedMap {
            window,
            event_loop,
            scheduler,
            http_client: self.http_client.unwrap(),
            style,
        }
    }
}
