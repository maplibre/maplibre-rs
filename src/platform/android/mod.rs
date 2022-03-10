use crate::io::workflow::Workflow;
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

    let mut workflow = Workflow::create();
    let download_tessellate_loop = workflow.take_download_loop();

    let join_handle = task::spawn_blocking(move || {
        Handle::current().block_on(async move {
            download_tessellate_loop.run_loop().await;
        });
    });

    main_loop::setup(window, event_loop, Box::new(workflow)).await;
    join_handle.await.unwrap()
}
