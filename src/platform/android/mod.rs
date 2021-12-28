#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
fn main() {
    use winit::event_loop::EventLoop;
    use winit::window::WindowBuilder;

    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("A fantastic window!")
        .build(&event_loop)
        .unwrap();

    pollster::block_on(crate::main_loop::setup(window, event_loop));
}
