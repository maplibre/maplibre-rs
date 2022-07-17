//! # Maplibre-rs
//!
//! A multi-platform library for rendering vector tile maps with WebGPU.
//!
//! Maplibre-rs is a map renderer that can run natively on MacOS, Linux, Windows, Android, iOS and the web.
//! It takes advantage of Lyon to tessellate vector tiles and WebGPU to display them efficiently.
//! Maplibre-rs also has an headless mode (*work in progress*) that can generate rasters.
//!
//! The official guide book can be found [here](https://maplibre.org/maplibre-rs/docs/book/).
//!
//! ### Example
//!
//! To import maplibre-rs in your `Cargo.toml`:
//!
//! ```toml
//! maplibre = "0.0.2"
//! ```

use crate::{
    io::{
        scheduler::{ScheduleMethod, Scheduler},
        source_client::HttpClient,
    },
    map_schedule::InteractiveMapSchedule,
    render::{
        settings::{RendererSettings, WgpuSettings},
        RenderState, Renderer,
    },
    style::Style,
    window::{EventLoop, HeadedMapWindow, MapWindow, MapWindowConfig, WindowSize},
};

pub mod context;
pub mod coords;
pub mod error;
#[cfg(feature = "headless")]
pub mod headless;
pub mod io;
// Exposed because of input handlers in maplibre-winit
pub mod map_schedule;
pub mod platform;
// Exposed because of camera
pub mod render;
pub mod style;
pub mod util;

pub mod window;
// Exposed because of doc-strings
pub mod schedule;
// Exposed because of SharedThreadState
pub mod stages;

// Used for benchmarking
pub mod benchmarking;

// Internal modules
pub(crate) mod tessellation;

/// The [`Map`] defines the public interface of the map renderer.
// DO NOT IMPLEMENT INTERNALS ON THIS STRUCT.
pub struct Map<MWC, SM, HC>
where
    MWC: MapWindowConfig,
    SM: ScheduleMethod,
    HC: HttpClient,
{
    map_schedule: InteractiveMapSchedule<MWC, SM, HC>,
    window: MWC::MapWindow,
}

impl<MWC, SM, HC> Map<MWC, SM, HC>
where
    MWC: MapWindowConfig,
    SM: ScheduleMethod,
    HC: HttpClient,
{
    /// Starts the [`crate::map_schedule::MapState`] Runnable with the configured event loop.
    pub fn run(self)
    where
        MWC::MapWindow: EventLoop<MWC, SM, HC>,
    {
        self.run_with_optionally_max_frames(None);
    }

    /// Starts the [`crate::map_schedule::MapState`] Runnable with the configured event loop.
    ///
    /// # Arguments
    ///
    /// * `max_frames` - Maximum number of frames per second.
    pub fn run_with_max_frames(self, max_frames: u64)
    where
        MWC::MapWindow: EventLoop<MWC, SM, HC>,
    {
        self.run_with_optionally_max_frames(Some(max_frames));
    }

    /// Starts the MapState Runnable with the configured event loop.
    ///
    /// # Arguments
    ///
    /// * `max_frames` - Optional maximum number of frames per second.
    pub fn run_with_optionally_max_frames(self, max_frames: Option<u64>)
    where
        MWC::MapWindow: EventLoop<MWC, SM, HC>,
    {
        self.window.run(self.map_schedule, max_frames);
    }

    pub fn map_schedule(&self) -> &InteractiveMapSchedule<MWC, SM, HC> {
        &self.map_schedule
    }

    pub fn map_schedule_mut(&mut self) -> &mut InteractiveMapSchedule<MWC, SM, HC> {
        &mut self.map_schedule
    }
}

/// Stores the map configuration before the map's state has been fully initialized.
pub struct UninitializedMap<MWC, SM, HC>
where
    MWC: MapWindowConfig,
    SM: ScheduleMethod,
    HC: HttpClient,
{
    scheduler: Scheduler<SM>,
    http_client: HC,
    style: Style,

    wgpu_settings: WgpuSettings,
    renderer_settings: RendererSettings,
    map_window_config: MWC,
}

impl<MWC, SM, HC> UninitializedMap<MWC, SM, HC>
where
    MWC: MapWindowConfig,
    SM: ScheduleMethod,
    HC: HttpClient,
{
    /// Initializes the whole rendering pipeline for the given configuration.
    /// Returns the initialized map, ready to be run.
    pub async fn initialize(self) -> Map<MWC, SM, HC>
    where
        MWC: MapWindowConfig,
        <MWC as MapWindowConfig>::MapWindow: HeadedMapWindow,
    {
        let window = self.map_window_config.create();
        let window_size = window.size();

        #[cfg(target_os = "android")]
        let renderer = None;
        #[cfg(not(target_os = "android"))]
        let renderer = Renderer::initialize(
            &window,
            self.wgpu_settings.clone(),
            self.renderer_settings.clone(),
        )
        .await
        .ok();
        Map {
            map_schedule: InteractiveMapSchedule::new(
                self.map_window_config,
                window_size,
                renderer,
                self.scheduler,
                self.http_client,
                self.style,
                self.wgpu_settings,
                self.renderer_settings,
            ),
            window,
        }
    }

    #[cfg(feature = "headless")]
    pub async fn initialize_headless(self) -> headless::HeadlessMap<MWC, SM, HC> {
        let window = self.map_window_config.create();
        let window_size = window.size();

        let renderer = Renderer::initialize_headless(
            &window,
            self.wgpu_settings.clone(),
            self.renderer_settings.clone(),
        )
        .await
        .expect("Failed to initialize renderer");
        headless::HeadlessMap {
            map_schedule: headless::HeadlessMapSchedule::new(
                self.map_window_config,
                window_size,
                renderer,
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
    wgpu_settings: Option<WgpuSettings>,
    renderer_settings: Option<RendererSettings>,
}

impl<MWC, SM, HC> MapBuilder<MWC, SM, HC>
where
    MWC: MapWindowConfig,
    SM: ScheduleMethod,
    HC: HttpClient,
{
    pub fn new() -> Self {
        Self {
            schedule_method: None,
            scheduler: None,
            http_client: None,
            style: None,
            map_window_config: None,
            wgpu_settings: None,
            renderer_settings: None,
        }
    }

    pub fn with_map_window_config(mut self, map_window_config: MWC) -> Self {
        self.map_window_config = Some(map_window_config);
        self
    }

    pub fn with_renderer_settings(mut self, renderer_settings: RendererSettings) -> Self {
        self.renderer_settings = Some(renderer_settings);
        self
    }

    pub fn with_wgpu_settings(mut self, wgpu_settings: WgpuSettings) -> Self {
        self.wgpu_settings = Some(wgpu_settings);
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

    /// Builds the UninitializedMap with the given configuration.
    pub fn build(self) -> UninitializedMap<MWC, SM, HC> {
        let scheduler = self
            .scheduler
            .unwrap_or_else(|| Scheduler::new(self.schedule_method.unwrap()));
        let style = self.style.unwrap_or_default();

        UninitializedMap {
            scheduler,
            http_client: self.http_client.unwrap(),
            style,
            wgpu_settings: self.wgpu_settings.unwrap_or_default(),
            renderer_settings: self.renderer_settings.unwrap_or_default(),
            map_window_config: self.map_window_config.unwrap(),
        }
    }
}
