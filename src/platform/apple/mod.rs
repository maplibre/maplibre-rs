use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use crate::io::worker_loop::DownloadTessellateLoop;
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

    let mut worker_loop = DownloadTessellateLoop::new();
    let worker_loop_main = worker_loop.clone();

    let join_handle = task::spawn(async move { worker_loop.run_loop().await });
    main_loop::setup(window, event_loop, Box::new(worker_loop_main), , ).await;
    join_handle.await.unwrap();
}
