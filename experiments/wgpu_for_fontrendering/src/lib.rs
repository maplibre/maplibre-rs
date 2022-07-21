mod rendering;
mod text_system;

use cgmath::One;
use cgmath::Quaternion;
use rand::Rng;
use std::time::{Duration, Instant};

use text_system::{FontID, SceneTextSystem};

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let size: winit::dpi::PhysicalSize<u32> = winit::dpi::PhysicalSize::new(4000, 2000);
    let window = WindowBuilder::new()
        .with_inner_size(size)
        .build(&event_loop)
        .unwrap();

    {
        let mut state: rendering::State = rendering::State::new(&window).await;
        let mut text_system: SceneTextSystem = SceneTextSystem::new(&state).unwrap();

        let font_id: FontID = String::from("Aparaj");
        if let Err(e) = text_system.load_font(&font_id, "tests/fonts/aparaj.ttf") {
            panic!("Couldn't add font!");
        }

        let step_x = 0.42;
        let step_y = 0.15;
        let z_jitter: f32 = 1.0;

        let limit = 30;

        let mut rng = rand::thread_rng();

        let title = format!("{} individual glyphs", (2 * limit) * (2 * limit) * 4);

        if let Err(_) = text_system.add_text_to_scene(
            &state,
            &title,
            (-0.8, (limit + 2) as f32 * step_y, 0.0).into(),
            Quaternion::new(0.0, 0.0, 0.0, 0.0),
            (1.0, 0.0, 0.0).into(),
            0.00015,
            &font_id,
        ) {
            panic!("Problem!");
        }

        for i in -limit..limit {
            for j in -limit..limit {
                let letter_1: char = rng.gen_range(b'A'..b'Z') as char;
                let letter_2: char = rng.gen_range(b'A'..b'Z') as char;
                let number: u32 = rng.gen_range(0..99);
                let text = format!("{}{}{:2}", letter_1, letter_2, number);

                let r: f32 = rng.gen_range(0.0..1.0);
                let g: f32 = rng.gen_range(0.0..1.0);
                let b: f32 = rng.gen_range(0.0..1.0);

                let z = rng.gen_range(-z_jitter..z_jitter);

                if let Err(_) = text_system.add_text_to_scene(
                    &state,
                    &text,
                    (i as f32 * step_x, j as f32 * step_y, z).into(),
                    Quaternion::new(0.0, 0.0, 0.0, 0.0),
                    (r, g, b).into(),
                    0.0001,
                    &font_id,
                ) {
                    panic!("Problem!");
                }
            }
        }

        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => {
                    if !state.input(event) {
                        match event {
                            WindowEvent::CloseRequested
                            | WindowEvent::KeyboardInput {
                                input:
                                    KeyboardInput {
                                        state: ElementState::Pressed,
                                        virtual_keycode: Some(VirtualKeyCode::Escape),
                                        ..
                                    },
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            WindowEvent::Resized(physical_size) => state.resize(*physical_size),
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                // new_inner_size is &&mut so we have to dereference it twice
                                state.resize(**new_inner_size);
                            }
                            _ => {}
                        }
                    }
                }
                Event::RedrawRequested(window_id) if window_id == window.id() => {
                    state.update();
                    let now = Instant::now();
                    match state.render(&mut text_system) {
                        Ok(_) => {}
                        // Reconfigure the surface if lost
                        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        // All other errors (Outdated, Timeout) should be resolved by the next frame
                        Err(e) => eprintln!("{:?}", e),
                    }
                    println!("Frame: {}ms", now.elapsed().as_millis());
                }
                Event::MainEventsCleared => {
                    // RedrawRequested will only trigger once, unless we manually
                    // request it.
                    window.request_redraw();
                }
                _ => {}
            }
        });
    }
}
