use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

mod fps_meter;
mod platform;
mod render;
mod io;
mod setup;

#[cfg(target_arch = "wasm32")]
mod web;

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("A fantastic window!")
        .build(&event_loop)
        .unwrap();

    pollster::block_on(setup::setup(window, event_loop));
}
