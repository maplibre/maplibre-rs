use mapr::main_loop;
use std::sync::mpsc::channel;
use std::thread;
use tokio::runtime::Handle;

use mapr::io::tile_cache::TileCache;
use mapr::io::web_tile_fetcher::WebTileFetcher;
use mapr::io::workflow::{DownloadTessellateLoop, TileRequestDispatcher, Workflow};
use mapr::io::{HttpFetcherConfig, TileFetcher};
use tokio::task;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("A fantastic window!")
        .build(&event_loop)
        .unwrap();

    let workflow = Workflow::create();
    let download_tessellate_loop = workflow.download_tessellate_loop;
    let tile_request_dispatcher = workflow.tile_request_dispatcher;
    let layer_result_receiver = workflow.layer_result_receiver;

    let join_handle = task::spawn_blocking(move || {
        Handle::current().block_on(async move {
            download_tessellate_loop.run_loop().await;
        });
    });

    main_loop::setup(
        window,
        event_loop,
        Box::new(tile_request_dispatcher),
        Box::new(layer_result_receiver),
        Box::new(TileCache::new()),
    )
    .await;
    join_handle.await.unwrap()
}
