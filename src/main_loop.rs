use crate::example::fetch_munich_tiles;
use log::{error, info, trace};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use crate::input::InputHandler;
use crate::io::cache::Cache;
use crate::platform::Instant;
use crate::render::state::State;

pub async fn setup(window: winit::window::Window, event_loop: EventLoop<()>, cache: Box<Cache>) {
    info!("== mapr ==");

    fetch_munich_tiles(cache.as_ref());

    let mut input = InputHandler::new();
    let mut maybe_state: Option<State> = if cfg!(target_os = "android") {
        None
    } else {
        Some(State::new(&window).await)
    };

    let mut last_render_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        /* FIXME:   On Android we need to initialize the surface on Event::Resumed. On desktop this
                    event is not fired and we can do surface initialization anytime. Clean this up.
        */
        #[cfg(target_os = "android")]
        if maybe_state.is_none() && event == Event::Resumed {
            use tokio::runtime::Handle;
            use tokio::task;

            let state = task::block_in_place(|| {
                Handle::current().block_on(async { State::new(&window).await })
            });
            maybe_state = Some(state);
            return;
        }

        if let Some(state) = maybe_state.as_mut() {
            match event {
                Event::DeviceEvent {
                    ref event,
                    .. // We're not using device_id currently
                } => {
                    trace!("{:?}", event);
                    input.device_input(event, state, &window);
                }

                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => {
                    if !input.window_input(event, state, &window) {
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
                    input.update_state(state, dt);
                    state.upload_tile_geometry(&cache);
                    match state.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            error!("Surface Lost");
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
        }
    });
}
