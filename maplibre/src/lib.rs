use crate::io::scheduler::Scheduler;

mod input;

pub(crate) mod coords;
pub(crate) mod error;
pub(crate) mod io;
pub(crate) mod map_state;
pub(crate) mod platform;
pub(crate) mod render;
pub(crate) mod style;
pub(crate) mod tessellation;
pub(crate) mod tilejson;
pub(crate) mod util;
pub(crate) mod winit;

// Used for benchmarking
pub mod benchmarking;
pub mod window;

use crate::map_state::{MapState, Runnable};
use crate::render::render_state::RenderState;
use crate::style::Style;
use crate::window::{WindowFactory, WindowSize};
pub use io::scheduler::ScheduleMethod;
pub use platform::schedule_method::*;

pub trait MapWindow {
    fn size(&self) -> Option<WindowSize>;
}

pub struct Map<W: MapWindow, E> {
    map_state: MapState<W>,
    event_loop: E,
}

impl<W: MapWindow, E> Map<W, E>
where
    MapState<W>: Runnable<E>,
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

pub struct UninitializedMap<W, E> {
    window: W,
    event_loop: E,
    scheduler: Scheduler,
    style: Style,
}

impl<W, E> UninitializedMap<W, E>
where
    W: MapWindow + raw_window_handle::HasRawWindowHandle,
{
    pub async fn initialize(self) -> Map<W, E> {
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
                self.style,
            ),
            event_loop: self.event_loop,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<W, E> UninitializedMap<W, E>
where
    W: MapWindow + raw_window_handle::HasRawWindowHandle,
    MapState<W>: Runnable<E>,
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

pub struct MapBuilder<W, E> {
    window_factory: Box<WindowFactory<W, E>>,
    schedule_method: Option<ScheduleMethod>,
    scheduler: Option<Scheduler>,
    style: Option<Style>,
}

impl<W, E> MapBuilder<W, E>
where
    MapState<W>: Runnable<E>,
    W: MapWindow + raw_window_handle::HasRawWindowHandle,
{
    pub(crate) fn new(create_window: Box<WindowFactory<W, E>>) -> Self {
        Self {
            window_factory: create_window,
            schedule_method: None,
            scheduler: None,
            style: None,
        }
    }

    pub fn with_schedule_method(mut self, schedule_method: ScheduleMethod) -> Self {
        self.schedule_method = Some(schedule_method);
        self
    }

    pub fn with_existing_scheduler(mut self, scheduler: Scheduler) -> Self {
        self.scheduler = Some(scheduler);
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    pub fn build(self) -> UninitializedMap<W, E> {
        let (window, event_loop) = (self.window_factory)();

        let scheduler = self
            .scheduler
            .unwrap_or_else(|| Scheduler::new(self.schedule_method.unwrap_or_default()));
        let style = self.style.unwrap_or_default();

        UninitializedMap {
            window,
            event_loop,
            scheduler,
            style,
        }
    }
}
