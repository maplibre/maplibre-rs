use crate::io::scheduler::IOScheduler;
use crate::main_loop;
pub use std::time::Instant;
use tokio::task;

pub const COLOR_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
#[tokio::main]
pub async fn main() {
    use winit::event_loop::EventLoop;
    use winit::window::WindowBuilder;

    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("A fantastic window!")
        .build(&event_loop)
        .unwrap();

    let mut scheduler = IOScheduler::new();
    let download_tessellate_loop = scheduler.take_download_loop();

    let join_handle = task::spawn_blocking(move || {
        Handle::current().block_on(async move {
            if let Err(e) = download_tessellate_loop.run_loop().await {
                error!("Worker loop errored {:?}", e)
            }
        });
    });

    main_loop::setup(window, event_loop, Box::new(scheduler)).await;
    join_handle.await.unwrap()
}
