use instant::Instant;
use maplibre::error::Error;
use maplibre::io::scheduler::ScheduleMethod;
use maplibre::io::source_client::HTTPClient;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::window::WindowBuilder;

use crate::input::{InputController, UpdateState};

use super::WinitEventLoop;
use super::WinitMapWindow;
use super::WinitWindow;
use maplibre::map_state::MapState;
use maplibre::window::{MapWindow, Runnable, WindowSize};
use winit::platform::web::WindowBuilderExtWebSys;

impl MapWindow for WinitMapWindow {
    type EventLoop = WinitEventLoop;
    type Window = WinitWindow;

    fn create() -> (Self, Self::EventLoop)
    where
        Self: Sized,
    {
        let event_loop = WinitEventLoop::new();

        let window: winit::window::Window = WindowBuilder::new()
            .with_canvas(Some(get_canvas("maplibre")))
            .build(&event_loop)
            .unwrap();

        let size = get_body_size().unwrap();
        window.set_inner_size(size);
        return (Self { inner: window }, event_loop);
    }

    fn size(&self) -> WindowSize {
        let size = self.inner.inner_size();
        let window_size =
            WindowSize::new(size.width, size.height).expect("failed to get window dimensions.");
        window_size
    }

    fn inner(&self) -> &Self::Window {
        &self.inner
    }
}

pub fn get_body_size() -> Option<winit::dpi::LogicalSize<i32>> {
    let web_window: web_sys::Window = web_sys::window().unwrap();
    let document = web_window.document().unwrap();
    let body = document.body().unwrap();
    Some(winit::dpi::LogicalSize {
        width: body.client_width(),
        height: body.client_height(),
    })
}

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
