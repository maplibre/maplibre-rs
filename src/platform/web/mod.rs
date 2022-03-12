mod http_fetcher;

use std::panic;

use log::error;
use log::info;
use log::Level;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::platform::web::WindowBuilderExtWebSys;
use winit::window::{Window, WindowBuilder};

use console_error_panic_hook;
pub use instant::Instant;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Window as WebSysWindow;

// WebGPU
#[cfg(not(feature = "web-webgl"))]
pub const COLOR_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

// WebGL
#[cfg(feature = "web-webgl")]
pub const COLOR_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

use crate::coords::{TileCoords, WorldTileCoords};
use crate::io::scheduler::{IOScheduler, ThreadLocalTessellatorState, TileResult};
use crate::io::tile_cache::TileCache;
pub use http_fetcher::PlatformHttpFetcher;
use style_spec::source::TileAdressingScheme;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    if let Err(_) = console_log::init_with_level(Level::Info) {
        // Failed to initialize logging. No need to log a message.
    }
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[wasm_bindgen()]
extern "C" {
    pub fn fetch_tile(url: &str, request_id: u32);
}

#[wasm_bindgen]
pub fn create_scheduler() -> *mut IOScheduler {
    let scheduler = Box::new(IOScheduler::create());
    let scheduler_ptr = Box::into_raw(scheduler);
    return scheduler_ptr;
}

#[wasm_bindgen]
pub fn new_tessellator_state(workflow_ptr: *mut IOScheduler) -> *mut ThreadLocalTessellatorState {
    let workflow: Box<IOScheduler> = unsafe { Box::from_raw(workflow_ptr) };
    let tessellator_state = Box::new(workflow.new_tessellator_state());
    let tessellator_state_ptr = Box::into_raw(tessellator_state);
    // Call forget such that workflow does not get deallocated
    std::mem::forget(workflow);
    return tessellator_state_ptr;
}

#[wasm_bindgen]
pub fn tessellate_layers(
    tessellator_state_ptr: *mut ThreadLocalTessellatorState,
    request_id: u32,
    data: Box<[u8]>,
) {
    let tessellator_state: Box<ThreadLocalTessellatorState> =
        unsafe { Box::from_raw(tessellator_state_ptr) };

    tessellator_state.tessellate_layers(request_id, data);

    // Call forget such that workflow does not get deallocated
    std::mem::forget(tessellator_state);
}

#[wasm_bindgen]
pub async fn run(workflow_ptr: *mut IOScheduler) {
    let workflow: Box<IOScheduler> = unsafe { Box::from_raw(workflow_ptr) };
    let event_loop = EventLoop::new();

    let web_window: WebSysWindow = web_sys::window().unwrap();
    let document = web_window.document().unwrap();
    let body = document.body().unwrap();
    let builder = WindowBuilder::new();
    let canvas: web_sys::HtmlCanvasElement = document
        .get_element_by_id("mapr")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();

    let window: Window = builder
        .with_canvas(Some(canvas))
        .build(&event_loop)
        .unwrap();

    window.set_inner_size(LogicalSize {
        width: body.client_width(),
        height: body.client_height(),
    });

    // Either call forget or the main loop to keep worker loop alive
    crate::main_loop::setup(window, event_loop, workflow).await;
    // std::mem::forget(workflow);
}
