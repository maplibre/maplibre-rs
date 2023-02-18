#![deny(unused_imports)]

use std::{fmt::Debug, marker::PhantomData};

use instant::Instant;
use maplibre::{
    environment::{Environment, OffscreenKernelEnvironment},
    event_loop::{EventLoop, EventLoopProxy, SendEventError},
    io::{apc::AsyncProcedureCall, scheduler::Scheduler, source_client::HttpClient},
    map::Map,
    window::{HeadedMapWindow, MapWindowConfig},
};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

use crate::input::{InputController, UpdateState};

pub mod input;

pub type RawWinitWindow = winit::window::Window;
pub type RawWinitEventLoop<ET> = winit::event_loop::EventLoop<ET>;
pub type RawEventLoopProxy<ET> = winit::event_loop::EventLoopProxy<ET>;

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(not(target_arch = "wasm32"))]
mod noweb;

#[cfg(not(target_arch = "wasm32"))]
pub use noweb::*;
#[cfg(target_arch = "wasm32")]
pub use web::*;

pub struct WinitMapWindow<ET: 'static> {
    window: RawWinitWindow,
    event_loop: Option<WinitEventLoop<ET>>,
}

impl<ET> WinitMapWindow<ET> {
    pub fn take_event_loop(&mut self) -> Option<WinitEventLoop<ET>> {
        self.event_loop.take()
    }
}

impl<ET> HeadedMapWindow for WinitMapWindow<ET> {
    type RawWindow = RawWinitWindow;

    fn raw(&self) -> &Self::RawWindow {
        &self.window
    }

    fn request_redraw(&self) {
        self.window.request_redraw()
    }

    fn id(&self) -> u64 {
        self.window.id().into()
    }
}

pub struct WinitEventLoop<ET: 'static> {
    event_loop: RawWinitEventLoop<ET>,
}

impl<ET: 'static + PartialEq + Debug> EventLoop<ET> for WinitEventLoop<ET> {
    type EventLoopProxy = WinitEventLoopProxy<ET>;

    fn run<E>(self, mut map: Map<E>, max_frames: Option<u64>)
    where
        E: Environment,
        <E::MapWindowConfig as MapWindowConfig>::MapWindow: HeadedMapWindow,
    {
        let mut last_render_time = Instant::now();
        let mut current_frame: u64 = 0;

        let mut input_controller = InputController::new(0.2, 100.0, 0.1);

        self.event_loop
            .run(move |event, _window_target, control_flow| {
                #[cfg(target_os = "android")]
                if !map.has_renderer() && event == Event::Resumed {
                    use tokio::{runtime::Handle, task};

                    task::block_in_place(|| {
                        Handle::current().block_on(async {
                            map.initialize_renderer().await.unwrap();
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
                    } if window_id == map.window().id().into() => {
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
                                    if let Ok(map_context) =  map.context_mut() {
                                        map_context.resize(physical_size.width, physical_size.height);
                                    }
                                }
                                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                    if let Ok(map_context) =  map.context_mut() {
                                        map_context.resize(new_inner_size.width, new_inner_size.height);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Event::RedrawRequested(_) => {
                        let now = Instant::now();
                        let dt = now - last_render_time;
                        last_render_time = now;

                        if let Ok(map_context) =  map.context_mut() {
                            input_controller.update_state(map_context, dt);
                        }

                        // TODO: Maybe handle gracefully
                        map.run_schedule().expect("Failed to run schedule!");

                        if let Some(max_frames) = max_frames {
                            if current_frame >= max_frames {
                                log::info!("Exiting because maximum frames reached.");
                                *control_flow = ControlFlow::Exit;
                            }

                            current_frame += 1;
                        }
                    }
                    Event::Suspended => {
                        // FIXME unimplemented!()
                    }
                    Event::Resumed => {
                        // FIXME unimplemented!()
                    }
                    Event::MainEventsCleared => {
                        // RedrawRequested will only trigger once, unless we manually
                        // request it.
                        map.window().request_redraw();
                    }
                    _ => {}
                }
            });
    }

    fn create_proxy(&self) -> Self::EventLoopProxy {
        WinitEventLoopProxy {
            proxy: self.event_loop.create_proxy(),
        }
    }
}
pub struct WinitEventLoopProxy<ET: 'static> {
    proxy: RawEventLoopProxy<ET>,
}

impl<ET: 'static> EventLoopProxy<ET> for WinitEventLoopProxy<ET> {
    fn send_event(&self, event: ET) -> Result<(), SendEventError> {
        self.proxy
            .send_event(event)
            .map_err(|_e| SendEventError::Closed)
    }
}

pub struct WinitEnvironment<
    S: Scheduler,
    HC: HttpClient,
    K: OffscreenKernelEnvironment,
    APC: AsyncProcedureCall<K>,
    ET,
> {
    phantom_s: PhantomData<S>,
    phantom_hc: PhantomData<HC>,
    phantom_k: PhantomData<K>,
    phantom_apc: PhantomData<APC>,
    phantom_et: PhantomData<ET>,
}

impl<
        S: Scheduler,
        HC: HttpClient,
        K: OffscreenKernelEnvironment,
        APC: AsyncProcedureCall<K>,
        ET: 'static,
    > Environment for WinitEnvironment<S, HC, K, APC, ET>
{
    type MapWindowConfig = WinitMapWindowConfig<ET>;
    type AsyncProcedureCall = APC;
    type Scheduler = S;
    type HttpClient = HC;
    type OffscreenKernelEnvironment = K;
}
