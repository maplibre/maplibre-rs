use std::{cell::RefCell, marker::PhantomData, ops::Deref, rc::Rc};

use instant::Instant;
use maplibre::{
    environment::Environment,
    error::Error,
    io::{
        apc::{AsyncProcedureCall, Message},
        scheduler::Scheduler,
        source_client::HttpClient,
        transferables::{DefaultTransferables, Transferables},
    },
    map_schedule::InteractiveMapSchedule,
    window::{EventLoop, HeadedMapWindow, MapWindowConfig},
};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

use crate::input::{InputController, UpdateState};

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(not(target_arch = "wasm32"))]
mod noweb;

#[cfg(not(target_arch = "wasm32"))]
pub use noweb::*;
#[cfg(target_arch = "wasm32")]
pub use web::*;

#[cfg(not(target_arch = "wasm32"))]
pub struct WinitMapWindowConfig {
    title: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl WinitMapWindowConfig {
    pub fn new(title: String) -> Self {
        Self { title }
    }
}

#[cfg(target_arch = "wasm32")]
pub struct WinitMapWindowConfig {
    canvas_id: String,
}

#[cfg(target_arch = "wasm32")]
impl WinitMapWindowConfig {
    pub fn new(canvas_id: String) -> Self {
        Self { canvas_id }
    }
}

pub struct WinitMapWindow {
    window: WinitWindow,
    event_loop: Option<WinitEventLoop>,
}

impl WinitMapWindow {
    pub fn take_event_loop(&mut self) -> Option<WinitEventLoop> {
        self.event_loop.take()
    }
}

pub type WinitWindow = winit::window::Window;
pub type WinitEventLoop = winit::event_loop::EventLoop<()>;

pub struct WinitEnvironment<
    S: Scheduler,
    HC: HttpClient,
    T: Transferables,
    APC: AsyncProcedureCall<T, HC>,
> {
    phantom_s: PhantomData<S>,
    phantom_hc: PhantomData<HC>,
    phantom_t: PhantomData<T>,
    phantom_apc: PhantomData<APC>,
}

impl<S: Scheduler, HC: HttpClient, T: Transferables, APC: AsyncProcedureCall<T, HC>> Environment
    for WinitEnvironment<S, HC, T, APC>
{
    type MapWindowConfig = WinitMapWindowConfig;
    type AsyncProcedureCall = APC;
    type Scheduler = S;
    type HttpClient = HC;
    type Transferables = T;
}

///Main (platform-specific) main loop which handles:
///* Input (Mouse/Keyboard)
///* Platform Events like suspend/resume
///* Render a new frame
impl<E: Environment> EventLoop<E> for WinitMapWindow
where
    E::MapWindowConfig: MapWindowConfig<MapWindow = WinitMapWindow>,
{
    fn run(
        mut self,
        map_schedule: Rc<RefCell<InteractiveMapSchedule<E>>>,
        max_frames: Option<u64>,
    ) {
        let mut last_render_time = Instant::now();
        let mut current_frame: u64 = 0;

        let mut input_controller = InputController::new(0.2, 100.0, 0.1);

        self.take_event_loop()
            .unwrap()
            .run(move |event, _, control_flow| {
                let mut map_schedule = map_schedule.deref().borrow_mut();

                #[cfg(target_os = "android")]
                if !map_schedule.is_initialized() && event == Event::Resumed {
                    use tokio::{runtime::Handle, task};

                    task::block_in_place(|| {
                        Handle::current().block_on(async {
                            map_schedule.late_init().await;
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
                                map_schedule.resize(physical_size.width, physical_size.height);
                            }
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                map_schedule.resize(new_inner_size.width, new_inner_size.height);
                            }
                            _ => {}
                        }
                    }
                }
                Event::RedrawRequested(_) => {
                    let now = Instant::now();
                    let dt = now - last_render_time;
                    last_render_time = now;

                    input_controller.update_state(map_schedule.view_state_mut(), dt);

                    match map_schedule.update_and_redraw() {
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
                    map_schedule.suspend();
                }
                Event::Resumed => {
                    map_schedule.resume(&self);
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
