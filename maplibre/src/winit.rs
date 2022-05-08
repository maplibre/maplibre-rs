//! Main (platform-specific) main loop which handles:
//! * Input (Mouse/Keyboard)
//! * Platform Events like suspend/resume
//! * Render a new frame

use instant::Instant;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use crate::input::{InputController, UpdateState};

use crate::map_state::{MapState, Runnable};

use crate::window::FromWindow;
use crate::{HTTPClient, MapBuilder, MapWindow, ScheduleMethod, WindowSize};

impl MapWindow for winit::window::Window {
    fn size(&self) -> Option<WindowSize> {
        let size = self.inner_size();
        WindowSize::new(size.width, size.height)
    }
}

impl<SM, HC> Runnable<winit::event_loop::EventLoop<()>> for MapState<winit::window::Window, SM, HC>
where
    SM: ScheduleMethod,
    HC: HTTPClient,
{
    fn run(mut self, event_loop: winit::event_loop::EventLoop<()>, max_frames: Option<u64>) {
        let mut last_render_time = Instant::now();
        let mut current_frame: u64 = 0;

        let mut input_controller = InputController::new(0.2, 100.0, 0.1);

        event_loop.run(move |event, _, control_flow| {

            #[cfg(target_os = "android")]
            if !self.is_initialized() && event == Event::Resumed {
                use tokio::runtime::Handle;
                use tokio::task;

                let state = task::block_in_place(|| {
                    Handle::current().block_on(async {
                        self.reinitialize().await;
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
                } if window_id == self.window().id() => {
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
                                self.resize(physical_size.width, physical_size.height);
                            }
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                self.resize(new_inner_size.width, new_inner_size.height);
                            }
                            _ => {}
                        }
                    }
                }
                Event::RedrawRequested(_) => {
                    let _span_ = tracing::span!(tracing::Level::TRACE, "redraw requested").entered();

                    let now = Instant::now();
                    let dt = now - last_render_time;
                    last_render_time = now;

                    // FIXME: Inputs are updated in RedrawRequested which
                    // 1. Adds a strain on the main thread, maybe we need another thread for that.
                    // 2. Delta time is correlated to the frame rate, and a low frame rate will cause
                    //    a lag with inputs. e.g. a button is held for a certain amount of time.
                    input_controller.update_state(self.view_state_mut(), dt);

                    match self.update_and_redraw() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            log::error!("Surface Lost");
                        }
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            log::error!("Out of Memory");
                            *control_flow = ControlFlow::Exit;
                        }
                        // All other errors (Outdated, Timeout) should be resolved by the next frame
                        Err(e) => eprintln!("{:?}", e),
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
                    self.suspend();
                }
                Event::Resumed => {
                    self.recreate_surface();
                    let size = self.window().inner_size();
                    self.resize(size.width, size.height);// FIXME: Resumed is also called when the app launches for the first time. Instead of first using a "fake" inner_size() in State::new we should initialize with a proper size from the beginning
                    self.resume();
                }
                Event::MainEventsCleared => {
                    // RedrawRequested will only trigger once, unless we manually
                    // request it.
                    self.window().request_redraw();
                }
                _ => {}
            }
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<SM, HC> FromWindow
    for MapBuilder<winit::window::Window, winit::event_loop::EventLoop<()>, SM, HC>
where
    SM: ScheduleMethod,
    HC: HTTPClient,
{
    fn from_window(title: &'static str) -> Self {
        let event_loop = EventLoop::new();
        Self::new(Box::new(move || {
            let window = WindowBuilder::new()
                .with_title(title)
                .build(&event_loop)
                .unwrap();
            (window, event_loop)
        }))
    }
}

#[cfg(target_arch = "wasm32")]
pub fn get_body_size() -> Option<winit::dpi::LogicalSize<i32>> {
    let web_window: web_sys::Window = web_sys::window().unwrap();
    let document = web_window.document().unwrap();
    let body = document.body().unwrap();
    Some(winit::dpi::LogicalSize {
        width: body.client_width(),
        height: body.client_height(),
    })
}

#[cfg(target_arch = "wasm32")]
pub fn get_canvas(element_id: &'static str) -> web_sys::HtmlCanvasElement {
    use wasm_bindgen::JsCast;

    let web_window: web_sys::Window = web_sys::window().unwrap();
    let document = web_window.document().unwrap();
    document
        .get_element_by_id(element_id)
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap()
}

#[cfg(target_arch = "wasm32")]
impl<SM, HC> crate::window::FromCanvas
    for MapBuilder<winit::window::Window, winit::event_loop::EventLoop<()>, SM, HC>
where
    SM: ScheduleMethod,
    HC: HTTPClient,
{
    fn from_canvas(dom_id: &'static str) -> Self {
        let event_loop = EventLoop::new();
        Self::new(Box::new(move || {
            use winit::platform::web::WindowBuilderExtWebSys;

            let window: winit::window::Window = WindowBuilder::new()
                .with_canvas(Some(get_canvas(dom_id)))
                .build(&event_loop)
                .unwrap();

            let size = get_body_size().unwrap();
            window.set_inner_size(size);
            (window, event_loop)
        }))
    }
}
