use std::panic;

use crate::style::source::TileAddressingScheme;
use console_error_panic_hook;
pub use instant::Instant;
use schedule_method::WebWorkerPoolScheduleMethod;
use wasm_bindgen::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Window as WebSysWindow;
use web_sys::Worker;

use crate::io::scheduler::ScheduleMethod;
use crate::io::scheduler::Scheduler;
use crate::window::FromCanvas;
use crate::MapBuilder;

pub mod http_client;
pub mod legacy_webworker_fetcher;
mod pool;
pub mod schedule_method;

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

#[wasm_bindgen]
pub async fn run(scheduler_ptr: *mut Scheduler) {
    let scheduler: Box<Scheduler> = unsafe { Box::from_raw(scheduler_ptr) };

    // Either call forget or the main loop to keep worker loop alive
    MapBuilder::from_canvas("maplibre")
        .with_existing_scheduler(*scheduler)
        .build()
        .initialize()
        .await
        .run();

    // std::mem::forget(scheduler);
}
