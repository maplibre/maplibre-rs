use std::marker::PhantomData;

use maplibre::window::{MapWindow, MapWindowConfig, WindowSize};
use winit::{platform::web::WindowBuilderExtWebSys, window::WindowBuilder};

use super::WinitMapWindow;
use crate::WinitEventLoop;

pub struct WinitMapWindowConfig<ET> {
    canvas_id: String,
    phantom_et: PhantomData<ET>,
}

impl<ET: 'static> WinitMapWindowConfig<ET> {
    pub fn new(canvas_id: String) -> Self {
        Self {
            canvas_id,
            phantom_et: Default::default(),
        }
    }
}

impl<ET: 'static> MapWindowConfig for WinitMapWindowConfig<ET> {
    type MapWindow = WinitMapWindow<ET>;

    fn create(&self) -> Self::MapWindow {
        let raw_event_loop = winit::event_loop::EventLoopBuilder::<ET>::with_user_event().build();

        let window: winit::window::Window = WindowBuilder::new()
            .with_canvas(Some(get_canvas(&self.canvas_id)))
            .build(&raw_event_loop)
            .unwrap();

        let size = get_body_size().unwrap();
        window.set_inner_size(size);
        Self::MapWindow {
            window,
            event_loop: Some(WinitEventLoop {
                event_loop: raw_event_loop,
            }),
        }
    }
}

impl<ET: 'static> MapWindow for WinitMapWindow<ET> {
    fn size(&self) -> WindowSize {
        let size = self.window.inner_size();

        WindowSize::new(size.width, size.height).expect("failed to get window dimensions.")
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

pub fn get_canvas(element_id: &str) -> web_sys::HtmlCanvasElement {
    use wasm_bindgen::JsCast;

    let web_window: web_sys::Window = web_sys::window().unwrap();
    let document = web_window.document().unwrap();
    document
        .get_element_by_id(element_id)
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap()
}
