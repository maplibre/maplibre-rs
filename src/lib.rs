use crate::io::scheduler::IOScheduler;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

mod input;

pub(crate) mod coords;
pub(crate) mod error;
pub(crate) mod io;
pub(crate) mod main_loop;
pub(crate) mod platform;
pub(crate) mod render;
pub(crate) mod tessellation;
pub(crate) mod util;

// Used for benchmarking
pub mod benchmarking;

pub use io::scheduler::ScheduleMethod;
pub use platform::scheduler::*;
use style_spec::Style;

pub struct Map {
    style: Style,
    window: winit::window::Window,
    event_loop: EventLoop<()>,
    scheduler: Box<IOScheduler>,
}

impl Map {
    #[cfg(target_arch = "wasm32")]
    pub async fn run_async(self) {
        main_loop::run(
            self.window,
            self.event_loop,
            self.scheduler,
            Box::new(self.style),
            None,
        )
        .await;
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn run_sync(self) {
        self.run_sync_with_max_frames(None);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn run_sync_with_max_frames(self, max_frames: Option<u64>) {
        tokio::runtime::Builder::new_multi_thread()
            .enable_io()
            .build()
            .unwrap()
            .block_on(async {
                main_loop::run(
                    self.window,
                    self.event_loop,
                    self.scheduler,
                    Box::new(self.style),
                    max_frames,
                )
                .await;
            })
    }
}

pub struct MapBuilder {
    create_window: Box<dyn FnOnce(&EventLoop<()>) -> winit::window::Window>,
    schedule_method: Option<ScheduleMethod>,
    scheduler: Option<Box<IOScheduler>>,
    style: Option<Style>,
}

impl MapBuilder {
    pub fn with_schedule_method(mut self, schedule_method: ScheduleMethod) -> Self {
        self.schedule_method = Some(schedule_method);
        self
    }

    pub fn with_existing_scheduler(mut self, scheduler: Box<IOScheduler>) -> Self {
        self.scheduler = Some(scheduler);
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_window(title: &'static str) -> Self {
        Self {
            create_window: Box::new(move |event_loop| {
                WindowBuilder::new()
                    .with_title(title)
                    .build(event_loop)
                    .unwrap()
            }),
            schedule_method: None,
            scheduler: None,
            style: None,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn from_canvas(dom_id: &'static str) -> Self {
        Self {
            create_window: Box::new(move |event_loop| {
                use crate::platform::{get_body_size, get_canvas};
                use winit::platform::web::WindowBuilderExtWebSys;

                let window: winit::window::Window = WindowBuilder::new()
                    .with_canvas(Some(get_canvas(dom_id)))
                    .build(&event_loop)
                    .unwrap();

                window.set_inner_size(get_body_size().unwrap());
                window
            }),
            schedule_method: None,
            scheduler: None,
            style: None,
        }
    }

    pub fn build(self) -> Map {
        let event_loop = EventLoop::new();

        Map {
            style: self.style.unwrap_or_default(),
            window: (self.create_window)(&event_loop),
            event_loop,
            scheduler: self.scheduler.unwrap_or_else(|| {
                Box::new(IOScheduler::new(self.schedule_method.unwrap_or_default()))
            }),
        }
    }
}
