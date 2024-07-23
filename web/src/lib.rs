#![deny(unused_imports)]

use maplibre::{
    environment::{OffscreenKernel, OffscreenKernelConfig},
    event_loop::EventLoop,
    io::source_client::{HttpSourceClient, SourceClient},
    kernel::{Kernel, KernelBuilder},
    map::Map,
    render::builder::RendererBuilder,
    style::Style,
};
use maplibre_winit::{WinitEnvironment, WinitMapWindowConfig};
use wasm_bindgen::prelude::*;

use crate::{
    error::JSError,
    platform::{http_client::WHATWGFetchHttpClient, UsedOffscreenKernelEnvironment},
};

mod error;
mod platform;

#[cfg(not(any(no_pendantic_os_check, target_arch = "wasm32")))]
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

pub struct WHATWGOffscreenKernelEnvironment;

impl OffscreenKernel for WHATWGOffscreenKernelEnvironment {
    type HttpClient = WHATWGFetchHttpClient;

    fn create(config: OffscreenKernelConfig) -> Self {
        WHATWGOffscreenKernelEnvironment
    }

    fn source_client(&self) -> SourceClient<Self::HttpClient> {
        SourceClient::new(HttpSourceClient::new(WHATWGFetchHttpClient::default()))
    }
}

#[cfg(not(target_feature = "atomics"))]
type CurrentEnvironment = WinitEnvironment<
    maplibre::io::scheduler::NopScheduler,
    WHATWGFetchHttpClient,
    UsedOffscreenKernelEnvironment,
    platform::singlethreaded::apc::PassingAsyncProcedureCall,
    (),
>;

#[cfg(target_feature = "atomics")]
type CurrentEnvironment = WinitEnvironment<
    platform::multithreaded::pool_scheduler::WebWorkerPoolScheduler,
    WHATWGFetchHttpClient,
    UsedOffscreenKernelEnvironment,
    maplibre::io::apc::SchedulerAsyncProcedureCall<
        UsedOffscreenKernelEnvironment,
        platform::multithreaded::pool_scheduler::WebWorkerPoolScheduler,
    >,
    (),
>;

pub type MapType = Map<CurrentEnvironment>;

#[wasm_bindgen]
pub async fn run_maplibre(new_worker: js_sys::Function) -> Result<(), JSError> {
    let mut kernel_builder = KernelBuilder::new()
        .with_map_window_config(WinitMapWindowConfig::new("maplibre".to_string()))
        .with_http_client(WHATWGFetchHttpClient::default());

    let offscreen_kernel_config = OffscreenKernelConfig {
        cache_directory: None,
    };

    #[cfg(target_feature = "atomics")]
    {
        kernel_builder = kernel_builder
            .with_apc(maplibre::io::apc::SchedulerAsyncProcedureCall::new(
                platform::multithreaded::pool_scheduler::WebWorkerPoolScheduler::new(
                    new_worker.clone(),
                )?,
                offscreen_kernel_config,
            ))
            .with_scheduler(
                platform::multithreaded::pool_scheduler::WebWorkerPoolScheduler::new(new_worker)?,
            );
    }

    #[cfg(not(target_feature = "atomics"))]
    {
        kernel_builder = kernel_builder
            .with_apc(
                platform::singlethreaded::apc::PassingAsyncProcedureCall::new(
                    new_worker,
                    4,
                    offscreen_kernel_config,
                )?,
            )
            .with_scheduler(maplibre::io::scheduler::NopScheduler);
    }

    let kernel: Kernel<WinitEnvironment<_, _, UsedOffscreenKernelEnvironment, _, ()>> =
        kernel_builder.build();

    let mut map: MapType = Map::new(
        Style::default(),
        kernel,
        RendererBuilder::new(),
        vec![
            Box::<maplibre::render::RenderPlugin>::default(),
            Box::<maplibre::vector::VectorPlugin<platform::UsedVectorTransferables>>::default(),
            // Box::new(RasterPlugin::<platform::UsedRasterTransferables>::default()),
            #[cfg(debug_assertions)]
            Box::<maplibre::debug::DebugPlugin>::default(),
        ],
    )
    .unwrap();
    map.initialize_renderer().await.unwrap();

    map.window_mut()
        .take_event_loop()
        .expect("Event loop is not available")
        .run(map, None);

    Ok(())
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
