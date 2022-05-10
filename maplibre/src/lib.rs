use crate::io::scheduler::{ScheduleMethod, Scheduler};
use crate::io::source_client::HTTPClient;
use crate::map_state::MapState;
use crate::render::render_state::RenderState;
use crate::style::Style;
use crate::window::{MapWindow, MapWindowConfig, Runnable, WindowSize};
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
pub mod map_state;
pub mod render;
pub(crate) mod tessellation;
pub(crate) mod tilejson;
pub(crate) mod util;

pub struct Map<W, SM, HC>
where
    W: MapWindow,
    SM: ScheduleMethod,
    HC: HTTPClient,
{
    map_state: MapState<W::MapWindowConfig, SM, HC>,
    window: W,
}

impl<W, SM, HC> Map<W, SM, HC>
where
    W: MapWindow + Runnable<W::MapWindowConfig, SM, HC>,
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
        self.window.run(self.map_state, max_frames);
    }
}

pub struct UninitializedMap<MWC, SM, HC>
where
    MWC: MapWindowConfig,
    SM: ScheduleMethod,
    HC: HTTPClient,
{
    scheduler: Scheduler<SM>,
    http_client: HC,
    style: Style,

    map_window_config: MWC,
}

impl<MWC, SM, HC> UninitializedMap<MWC, SM, HC>
where
    MWC: MapWindowConfig,
    SM: ScheduleMethod,
    HC: HTTPClient,
{
    pub async fn initialize(self) -> Map<MWC::MapWindow, SM, HC> {
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

        let render_state = RenderState::initialize(instance, surface, surface_config).await;
        Map {
            map_state: MapState::new(
                self.map_window_config,
                window_size,
                render_state,
                self.scheduler,
                self.http_client,
                self.style,
            ),
            window,
        }
    }
}

pub struct MapBuilder<MWC, SM, HC>
where
    SM: ScheduleMethod,
{
    schedule_method: Option<SM>,
    scheduler: Option<Scheduler<SM>>,
    http_client: Option<HC>,
    style: Option<Style>,

    map_window_config: Option<MWC>,
}

impl<MWC, SM, HC> MapBuilder<MWC, SM, HC>
where
    MWC: MapWindowConfig,
    SM: ScheduleMethod,
    HC: HTTPClient,
{
    pub fn new() -> Self {
        Self {
            schedule_method: None,
            scheduler: None,
            http_client: None,
            style: None,
            map_window_config: None,
        }
    }

    pub fn with_map_window_config(mut self, map_window_config: MWC) -> Self {
        self.map_window_config = Some(map_window_config);
        self
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

    pub fn build(self) -> UninitializedMap<MWC, SM, HC> {
        let scheduler = self
            .scheduler
            .unwrap_or_else(|| Scheduler::new(self.schedule_method.unwrap()));
        let style = self.style.unwrap_or_default();

        UninitializedMap {
            scheduler,
            http_client: self.http_client.unwrap(),
            style,
            map_window_config: self.map_window_config.unwrap(),
        }
    }
}
