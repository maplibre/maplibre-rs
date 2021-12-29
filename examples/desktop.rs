use mapr::io::pool::Pool;
use mapr::main_loop;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("A fantastic window!")
        .build(&event_loop)
        .unwrap();

    let io_tile_pool = Pool::new();
    let main_tile_pool = io_tile_pool.clone();

    std::thread::spawn(move || {
        io_tile_pool.run_loop();
    });

    pollster::block_on(main_loop::setup(window, event_loop, main_tile_pool));
}
