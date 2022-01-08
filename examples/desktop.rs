use futures::poll;
use mapr::io::cache::Cache;
use mapr::main_loop;
use pollster::FutureExt;
use tokio;
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

    let mut cache_io = Cache::new();
    let cache_main = cache_io.clone();

    let join_handle = task::spawn(async move { cache_io.run_loop().await });
    main_loop::setup(window, event_loop, Box::new(cache_main)).await;
    join_handle.await;
}
