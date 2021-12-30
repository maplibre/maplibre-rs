use log::{error, info, trace};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use crate::input::InputHandler;
use crate::io::cache::Cache;
use crate::platform::Instant;
use crate::render::state::State;

pub async fn setup(window: winit::window::Window, event_loop: EventLoop<()>, cache: Cache) {
    info!("== mapr ==");

    for x in 0..2 {
        for y in 0..2 {
            cache.fetch((2179 + x, 1421 + y, 12).into())
        }
    }

    let mut input = InputHandler::new();
    let mut state = State::new(&window).await;

    let mut last_render_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::DeviceEvent {
                ref event,
                .. // We're not using device_id currently
            } => {
                trace!("{:?}", event);
                input.device_input(event,&window);
            }

            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !input.window_input(event, &window) {
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
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &mut so w have to dereference it twice
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                let now = Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;
                input.update_state(&mut state, dt);
                state.upload_tile_geometry(&cache);
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => {
                        error!("Surface Lost");
                        *control_flow = ControlFlow::Exit;
                    },
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        error!("Out of Memory");
                        *control_flow = ControlFlow::Exit;
                    },
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::Suspended => {
                state.suspend();
            }
            Event::Resumed => {
                state.recreate_surface(&window);
                state.resize(window.inner_size()); // FIXME: Resumed is also called when the app launches for the first time. Instead of first using a "fake" inner_size() in State::new we should initialize with a proper size from the beginning
                state.resume();
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
