use crate::{
    headless::{environment::HeadlessEnvironment, window::HeadlessMapWindowConfig},
    io::apc::SchedulerAsyncProcedureCall,
    kernel::{Kernel, KernelBuilder},
    platform::{http_client::ReqwestHttpClient, scheduler::TokioScheduler},
    render::{builder::RenderBuilder, Renderer},
    window::WindowSize,
};

mod graph_node;
mod stage;

pub mod environment;
pub mod map;
pub mod window;

pub async fn create_headless_renderer(
    tile_size: u32,
    cache_path: Option<String>,
) -> (Kernel<HeadlessEnvironment>, Renderer) {
    let client = ReqwestHttpClient::new(cache_path);
    let kernel = KernelBuilder::new()
        .with_map_window_config(HeadlessMapWindowConfig::new(
            WindowSize::new(tile_size, tile_size).unwrap(),
        ))
        .with_http_client(client.clone())
        .with_apc(SchedulerAsyncProcedureCall::new(
            client,
            TokioScheduler::new(),
        ))
        .with_scheduler(TokioScheduler::new())
        .build();

    let renderer = RenderBuilder::new()
        .build()
        .initialize_headless_with(&kernel)
        .await
        .expect("Failed to initialize renderer");

    (kernel, renderer)
}
