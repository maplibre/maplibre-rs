use std::panic;

use maplibre::{io::scheduler::Scheduler, MapBuilder};
use maplibre_winit::winit::WinitMapWindowConfig;
use wasm_bindgen::prelude::*;

use crate::platform::{
    http_client::WHATWGFetchHttpClient, schedule_method::WebWorkerPoolScheduleMethod,
};

mod error;
mod platform;

#[cfg(not(target_arch = "wasm32"))]
compile_error!("web works only on wasm32.");

#[cfg(feature = "trace")]
fn enable_tracing() {
    use tracing_subscriber::{layer::SubscriberExt, Registry};

    let mut builder = tracing_wasm::WASMLayerConfigBuilder::new();
    builder.set_report_logs_in_timings(true);
    builder.set_console_config(tracing_wasm::ConsoleConfig::NoReporting);

    tracing::subscriber::set_global_default(
        Registry::default().with(tracing_wasm::WASMLayer::new(builder.build())),
    );
}

#[wasm_bindgen(start)]
pub fn wasm_bindgen_start() {
    if console_log::init_with_level(log::Level::Info).is_err() {
        // Failed to initialize logging. No need to log a message.
    }
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    #[cfg(any(feature = "trace"))]
    enable_tracing();
}

#[wasm_bindgen]
pub fn create_pool_scheduler(
    new_worker: js_sys::Function,
) -> *mut Scheduler<WebWorkerPoolScheduleMethod> {
    let scheduler = Box::new(Scheduler::new(WebWorkerPoolScheduleMethod::new(new_worker)));

    Box::into_raw(scheduler)
}

#[wasm_bindgen]
pub async fn run(scheduler_ptr: *mut Scheduler<WebWorkerPoolScheduleMethod>) {
    let scheduler: Box<Scheduler<WebWorkerPoolScheduleMethod>> =
        unsafe { Box::from_raw(scheduler_ptr) };

    // Either call forget or the main loop to keep worker loop alive
    MapBuilder::new()
        .with_map_window_config(WinitMapWindowConfig::new("maplibre".to_string()))
        .with_http_client(WHATWGFetchHttpClient::new())
        .with_existing_scheduler(*scheduler)
        .build()
        .initialize()
        .await
        .run();

    // std::mem::forget(scheduler);
}

#[cfg(test)]
/// See https://rustwasm.github.io/wasm-bindgen/wasm-bindgen-test/browsers.html
mod tests {
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn pass() {
        assert_eq!(1, 1);
    }
}
