use instant::Instant;
use maplibre::error::Error;
use maplibre::io::scheduler::ScheduleMethod;
use maplibre::io::source_client::HTTPClient;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::ControlFlow;

use crate::input::{InputController, UpdateState};
use maplibre::map_state::MapState;
use maplibre::window::{MapWindow, MapWindowConfig, Runnable};
use winit::event::Event;

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(not(target_arch = "wasm32"))]
mod noweb;

#[cfg(target_arch = "wasm32")]
pub use web::*;

#[cfg(not(target_arch = "wasm32"))]
pub use noweb::*;

pub struct WinitMapWindowConfig {
    title: String,
}

impl WinitMapWindowConfig {
    pub fn new(title: String) -> Self {
        Self { title }
    }
}

impl MapWindowConfig for WinitMapWindowConfig {
    type WindowMap = WinitMapWindow;
}

pub struct WinitMapWindow {
    window: WinitWindow,
    event_loop: Option<WinitEventLoop>,
}

pub type WinitWindow = winit::window::Window;
pub type WinitEventLoop = winit::event_loop::EventLoop<()>;

impl WinitMapWindow {
    pub fn take_event_loop(&mut self) -> Option<WinitEventLoop> {
        self.event_loop.take()
    }
}

///Main (platform-specific) main loop which handles:
///* Input (Mouse/Keyboard)
///* Platform Events like suspend/resume
///* Render a new frame
impl<SM, HC> Runnable<SM, HC> for WinitMapWindow
where
    SM: ScheduleMethod,
    HC: HTTPClient,
{
    fn run(mut self, mut map_state: MapState<SM, HC>, max_frames: Option<u64>) {
        let mut last_render_time = Instant::now();
        let mut current_frame: u64 = 0;

        let mut input_controller = InputController::new(0.2, 100.0, 0.1);

        self.take_event_loop()
            .unwrap()
            .run(move |event, _, control_flow| {
                #[cfg(target_os = "android")]
                if !map_state.is_initialized() && event == Event::Resumed {
                    use tokio::runtime::Handle;
                    use tokio::task;

                    let state = task::block_in_place(|| {
                        Handle::current().block_on(async {
                            map_state.reinitialize::<WinitMapWindow>().await;
                        })
                    });
                    return;
                }

                match event {
                Event::DeviceEvent {
                    ref event,
                    .. // We're not using device_id currently
                } => {
                    input_controller.device_input(event);
                }

                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.inner().id() => {
                    if !input_controller.window_input(event) {
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
                                map_state.resize(physical_size.width, physical_size.height);
                            }
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                map_state.resize(new_inner_size.width, new_inner_size.height);
                            }
                            _ => {}
                        }
                    }
                }
                Event::RedrawRequested(_) => {
                    let now = Instant::now();
                    let dt = now - last_render_time;
                    last_render_time = now;

                    input_controller.update_state(map_state.view_state_mut(), dt);

                    match map_state.update_and_redraw() {
                        Ok(_) => {}
                        Err(Error::Render(e)) => {
                            eprintln!("{}", e);
                            if e.should_exit() {
                                *control_flow = ControlFlow::Exit;
                            }
                        }
                        e => eprintln!("{:?}", e)
                    };

                    if let Some(max_frames) = max_frames {
                        if current_frame >= max_frames {
                            log::info!("Exiting because maximum frames reached.");
                            *control_flow = ControlFlow::Exit;
                        }

                        current_frame += 1;
                    }
                }
                Event::Suspended => {
                    map_state.suspend();
                }
                Event::Resumed => {
                    map_state.recreate_surface(&self);
                    let size = self.size();
                    map_state.resize(size.width(), size.height());// FIXME: Resumed is also called when the app launches for the first time. Instead of first using a "fake" inner_size() in State::new we should initialize with a proper size from the beginning
                    map_state.resume();
                }
                Event::MainEventsCleared => {
                    // RedrawRequested will only trigger once, unless we manually
                    // request it.
                    self.inner().request_redraw();
                }
                _ => {}
            }
            });
    }
}
