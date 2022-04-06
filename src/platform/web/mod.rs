use std::panic;

use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::platform::web::WindowBuilderExtWebSys;
use winit::window::{Window, WindowBuilder};

use console_error_panic_hook;
pub use instant::Instant;
use schedule_method::WebWorkerPoolScheduleMethod;
use style_spec::source::TileAddressingScheme;
use wasm_bindgen::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Window as WebSysWindow;
use web_sys::Worker;

use crate::io::scheduler::ScheduleMethod;
use crate::io::scheduler::Scheduler;
use crate::io::scheduler::ThreadLocalState;
use crate::MapBuilder;

pub mod http_client;
pub mod legacy_webworker_fetcher;
mod pool;
pub mod schedule_method;

// WebGPU
#[cfg(not(feature = "web-webgl"))]
pub const COLOR_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

// WebGL
#[cfg(feature = "web-webgl")]
pub const COLOR_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

#[cfg(feature = "enable-tracing")]
fn enable_tracing() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::Registry;

    let mut builder = tracing_wasm::WASMLayerConfigBuilder::new();
    builder.set_report_logs_in_timings(true);
    builder.set_console_config(tracing_wasm::ConsoleConfig::NoReporting);

    tracing::subscriber::set_global_default(
        Registry::default().with(tracing_wasm::WASMLayer::new(builder.build())),
    );
}

#[wasm_bindgen(start)]
pub fn wasm_bindgen_start() {
    if let Err(_) = console_log::init_with_level(log::Level::Info) {
        // Failed to initialize logging. No need to log a message.
    }
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    #[cfg(any(feature = "enable-tracing"))]
    enable_tracing();
}

#[wasm_bindgen]
pub fn create_pool_scheduler(new_worker: js_sys::Function) -> *mut Scheduler {
    let scheduler = Box::new(Scheduler::new(ScheduleMethod::WebWorkerPool(
        WebWorkerPoolScheduleMethod::new(new_worker),
    )));
    let scheduler_ptr = Box::into_raw(scheduler);
    return scheduler_ptr;
}

pub fn get_body_size() -> Option<LogicalSize<i32>> {
    let web_window: WebSysWindow = web_sys::window().unwrap();
    let document = web_window.document().unwrap();
    let body = document.body().unwrap();
    Some(LogicalSize {
        width: body.client_width(),
        height: body.client_height(),
    })
}

pub fn get_canvas(element_id: &'static str) -> web_sys::HtmlCanvasElement {
    let web_window: WebSysWindow = web_sys::window().unwrap();
    let document = web_window.document().unwrap();
    document
        .get_element_by_id(element_id)
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap()
}

#[wasm_bindgen]
pub async fn run(scheduler_ptr: *mut Scheduler) {
    let scheduler: Box<Scheduler> = unsafe { Box::from_raw(scheduler_ptr) };

    // Either call forget or the main loop to keep worker loop alive
    MapBuilder::from_canvas("mapr")
        .with_existing_scheduler(scheduler)
        .build()
        .run_async()
        .await;

    // std::mem::forget(scheduler);
}
