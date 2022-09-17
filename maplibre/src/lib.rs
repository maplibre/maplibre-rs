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

use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    rc::Rc,
};

use crate::{
    environment::Environment,
    io::{scheduler::Scheduler, source_client::HttpClient},
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

pub mod environment;

pub use geozero::mvt::tile;

/// The [`Map`] defines the public interface of the map renderer.
// DO NOT IMPLEMENT INTERNALS ON THIS STRUCT.
pub struct Map<E: Environment> {
    // FIXME (wasm-executor): Avoid RefCell, change ownership model!
    map_schedule: Rc<RefCell<InteractiveMapSchedule<E>>>,
    window: RefCell<Option<<E::MapWindowConfig as MapWindowConfig>::MapWindow>>,
}

impl<E: Environment> Map<E>
where
    <E::MapWindowConfig as MapWindowConfig>::MapWindow: EventLoop<E>,
{
    /// Starts the [`crate::map_schedule::MapState`] Runnable with the configured event loop.
    pub fn run(&self) {
        self.run_with_optionally_max_frames(None);
    }

    /// Starts the [`crate::map_schedule::MapState`] Runnable with the configured event loop.
    ///
    /// # Arguments
    ///
    /// * `max_frames` - Maximum number of frames per second.
    pub fn run_with_max_frames(&self, max_frames: u64) {
        self.run_with_optionally_max_frames(Some(max_frames));
    }

    /// Starts the MapState Runnable with the configured event loop.
    ///
    /// # Arguments
    ///
    /// * `max_frames` - Optional maximum number of frames per second.
    pub fn run_with_optionally_max_frames(&self, max_frames: Option<u64>) {
        self.window
            .borrow_mut()
            .take()
            .unwrap() // FIXME (wasm-executor): Remove unwrap
            .run(self.map_schedule.clone(), max_frames);
    }

    pub fn map_schedule(&self) -> Rc<RefCell<InteractiveMapSchedule<E>>> {
        self.map_schedule.clone()
    }

    /*    pub fn map_schedule_mut(&mut self) -> &mut InteractiveMapSchedule<E> {
        &mut self.map_schedule
    }*/
}

/// Stores the map configuration before the map's state has been fully initialized.
pub struct UninitializedMap<E: Environment> {
    scheduler: E::Scheduler,
    apc: E::AsyncProcedureCall,
    http_client: E::HttpClient,
    style: Style,

    wgpu_settings: WgpuSettings,
    renderer_settings: RendererSettings,
    map_window_config: E::MapWindowConfig,
}

impl<E: Environment> UninitializedMap<E>
where
    <E::MapWindowConfig as MapWindowConfig>::MapWindow: HeadedMapWindow,
{
    /// Initializes the whole rendering pipeline for the given configuration.
    /// Returns the initialized map, ready to be run.
    pub async fn initialize(self) -> Map<E> {
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
            map_schedule: Rc::new(RefCell::new(InteractiveMapSchedule::new(
                self.map_window_config,
                window_size,
                renderer,
                self.scheduler,
                self.apc,
                self.http_client,
                self.style,
                self.wgpu_settings,
                self.renderer_settings,
            ))),
            window: RefCell::new(Some(window)),
        }
    }
}

#[cfg(feature = "headless")]
impl<E: Environment> UninitializedMap<E> {
    pub async fn initialize_headless(self) -> headless::HeadlessMap<E> {
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

pub struct MapBuilder<E: Environment> {
    scheduler: Option<E::Scheduler>,
    apc: Option<E::AsyncProcedureCall>,
    http_client: Option<E::HttpClient>,
    style: Option<Style>,

    map_window_config: Option<E::MapWindowConfig>,
    wgpu_settings: Option<WgpuSettings>,
    renderer_settings: Option<RendererSettings>,
}

impl<E: Environment> MapBuilder<E> {
    pub fn new() -> Self {
        Self {
            scheduler: None,
            apc: None,
            http_client: None,
            style: None,
            map_window_config: None,
            wgpu_settings: None,
            renderer_settings: None,
        }
    }

    pub fn with_map_window_config(mut self, map_window_config: E::MapWindowConfig) -> Self {
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

    pub fn with_scheduler(mut self, scheduler: E::Scheduler) -> Self {
        self.scheduler = Some(scheduler);
        self
    }

    pub fn with_apc(mut self, apc: E::AsyncProcedureCall) -> Self {
        self.apc = Some(apc);
        self
    }

    pub fn with_http_client(mut self, http_client: E::HttpClient) -> Self {
        self.http_client = Some(http_client);
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    /// Builds the UninitializedMap with the given configuration.
    pub fn build(self) -> UninitializedMap<E> {
        UninitializedMap {
            scheduler: self.scheduler.unwrap(),     // TODO: Remove unwrap
            apc: self.apc.unwrap(),                 // TODO: Remove unwrap
            http_client: self.http_client.unwrap(), // TODO: Remove unwrap
            style: self.style.unwrap_or_default(),
            wgpu_settings: self.wgpu_settings.unwrap_or_default(),
            renderer_settings: self.renderer_settings.unwrap_or_default(),
            map_window_config: self.map_window_config.unwrap(), // TODO: Remove unwrap
        }
    }
}
