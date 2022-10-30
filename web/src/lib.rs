#![feature(allocator_api, new_uninit)]

use std::{borrow::BorrowMut, cell::RefCell, mem, ops::Deref, rc::Rc};

use maplibre::{
    event_loop::EventLoop,
    io::{apc::SchedulerAsyncProcedureCall, scheduler::NopScheduler},
    kernel::{Kernel, KernelBuilder},
    map::Map,
    render::builder::{InitializedRenderer, RenderBuilder},
    style::Style,
};
use maplibre_winit::{WinitEnvironment, WinitMapWindowConfig};
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
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    #[cfg(any(feature = "trace"))]
    enable_tracing();
}

#[cfg(not(target_feature = "atomics"))]
type CurrentEnvironment = WinitEnvironment<
    NopScheduler,
    WHATWGFetchHttpClient,
    platform::singlethreaded::apc::PassingAsyncProcedureCall,
    (),
>;

#[cfg(target_feature = "atomics")]
type CurrentEnvironment = WinitEnvironment<
    platform::multithreaded::pool_scheduler::WebWorkerPoolScheduler,
    WHATWGFetchHttpClient,
    maplibre::io::apc::SchedulerAsyncProcedureCall<
        WHATWGFetchHttpClient,
        platform::multithreaded::pool_scheduler::WebWorkerPoolScheduler,
    >,
    (),
>;

pub type MapType = Map<CurrentEnvironment>;

pub struct InitResult {
    initialized: InitializedRenderer<WinitMapWindowConfig<()>>,
    kernel: Kernel<CurrentEnvironment>,
}

#[wasm_bindgen]
pub async fn init_maplibre(new_worker: js_sys::Function) -> u32 {
    let mut kernel_builder = KernelBuilder::new()
        .with_map_window_config(WinitMapWindowConfig::new("maplibre".to_string()))
        .with_http_client(WHATWGFetchHttpClient::new());

    #[cfg(target_feature = "atomics")]
    {
        kernel_builder = kernel_builder
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
        kernel_builder = kernel_builder
            .with_apc(platform::singlethreaded::apc::PassingAsyncProcedureCall::new(new_worker, 4))
            .with_scheduler(NopScheduler);
    }

    let kernel: Kernel<WinitEnvironment<_, _, _, ()>> = kernel_builder.build();

    Box::into_raw(Box::new(InitResult {
        initialized: RenderBuilder::new()
            .build()
            .initialize_with(&kernel)
            .await
            .expect("Failed to initialize renderer")
            .unwarp(),
        kernel,
    })) as u32
}

#[wasm_bindgen]
pub unsafe fn run(init_ptr: *mut InitResult) {
    let mut init_result = Box::from_raw(init_ptr);

    let InitializedRenderer {
        mut window,
        renderer,
    } = init_result.initialized;
    let map: MapType = Map::new(Style::default(), init_result.kernel, renderer).unwrap();

    window
        .take_event_loop()
        .expect("Event loop is not available")
        .run(window, map, None)
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
