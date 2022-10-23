#![feature(allocator_api, new_uninit)]

use std::{borrow::BorrowMut, cell::RefCell, mem, ops::Deref, panic, rc::Rc};

use maplibre::{io::scheduler::NopScheduler, Map, MapBuilder};
use maplibre_winit::winit::{WinitEnvironment, WinitMapWindowConfig};
use wasm_bindgen::prelude::*;

use crate::platform::http_client::WHATWGFetchHttpClient;

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

#[cfg(not(target_feature = "atomics"))]
pub type MapType = Map<
    WinitEnvironment<
        NopScheduler,
        WHATWGFetchHttpClient,
        platform::singlethreaded::transferables::LinearTransferables,
        platform::singlethreaded::apc::PassingAsyncProcedureCall,
    >,
>;

#[cfg(target_feature = "atomics")]
pub type MapType = Map<
    WinitEnvironment<
        platform::multithreaded::pool_scheduler::WebWorkerPoolScheduler,
        WHATWGFetchHttpClient,
        maplibre::io::transferables::DefaultTransferables,
        maplibre::io::apc::SchedulerAsyncProcedureCall<
            WHATWGFetchHttpClient,
            platform::multithreaded::pool_scheduler::WebWorkerPoolScheduler,
        >,
    >,
>;

#[wasm_bindgen]
pub async fn create_map(new_worker: js_sys::Function) -> u32 {
    // Either call forget or the main loop to keep worker loop alive
    let mut builder = MapBuilder::new()
        .with_map_window_config(WinitMapWindowConfig::new("maplibre".to_string()))
        .with_http_client(WHATWGFetchHttpClient::new());

    #[cfg(target_feature = "atomics")]
    {
        builder = builder
            .with_apc(maplibre::io::apc::SchedulerAsyncProcedureCall::new(
                WHATWGFetchHttpClient::new(),
                platform::multithreaded::pool_scheduler::WebWorkerPoolScheduler::new(
                    new_worker.clone(),
                ),
            ))
            .with_scheduler(
                platform::multithreaded::pool_scheduler::WebWorkerPoolScheduler::new(new_worker),
            );
    }

    #[cfg(not(target_feature = "atomics"))]
    {
        builder = builder
            .with_apc(platform::singlethreaded::apc::PassingAsyncProcedureCall::new(new_worker, 4))
            .with_scheduler(NopScheduler);
    }

    let map: MapType = builder.build().initialize().await;

    Rc::into_raw(Rc::new(RefCell::new(map))) as u32
}

#[wasm_bindgen]
pub unsafe fn clone_map(map_ptr: *const RefCell<MapType>) -> *const RefCell<MapType> {
    let mut map = Rc::from_raw(map_ptr);
    let rc = map.clone();
    let cloned = Rc::into_raw(rc);
    mem::forget(map);
    cloned
}

#[wasm_bindgen]
pub unsafe fn run(map_ptr: *const RefCell<MapType>) {
    let mut map = Rc::from_raw(map_ptr);

    map.deref().borrow().run();
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
