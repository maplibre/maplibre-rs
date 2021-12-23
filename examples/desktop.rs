use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use mapr::setup;

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("A fantastic window!")
        .build(&event_loop)
        .unwrap();

    pollster::block_on(setup::setup(window, event_loop));
}
