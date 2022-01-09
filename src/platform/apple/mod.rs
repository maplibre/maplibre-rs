use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use crate::io::cache::Cache;
use crate::main_loop;
pub use std::time::Instant;
use tokio::task;

// macOS and iOS (Metal)
pub const COLOR_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

#[no_mangle]
#[tokio::main]
pub async fn mapr_apple_main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("A fantastic window!")
        .build(&event_loop)
        .unwrap();

    let mut cache_io = Cache::new();
    let cache_main = cache_io.clone();

    let mut cache_io = Cache::new();
    let cache_main = cache_io.clone();

    let join_handle = task::spawn(async move { cache_io.run_loop().await });
    main_loop::setup(window, event_loop, Box::new(cache_main)).await;
    join_handle.await;
}
