use maplibre::window::{HeadedMapWindow, MapWindow, MapWindowConfig, WindowSize};
use winit::{platform::web::WindowBuilderExtWebSys, window::WindowBuilder};

use super::{WinitEventLoop, WinitMapWindow, WinitMapWindowConfig, WinitWindow};

impl MapWindowConfig for WinitMapWindowConfig {
    type MapWindow = WinitMapWindow;

    fn create(&self) -> Self::MapWindow {
        let event_loop = WinitEventLoop::new();

        let window: winit::window::Window = WindowBuilder::new()
            .with_canvas(Some(get_canvas(&self.canvas_id)))
            .build(&event_loop)
            .unwrap();

        let size = get_body_size().unwrap();
        window.set_inner_size(size);
        Self::MapWindow {
            window,
            event_loop: Some(event_loop),
        }
    }
}

impl MapWindow for WinitMapWindow {
    fn size(&self) -> WindowSize {
        let size = self.window.inner_size();

        WindowSize::new(size.width, size.height).expect("failed to get window dimensions.")
    }
}
impl HeadedMapWindow for WinitMapWindow {
    type RawWindow = WinitWindow;

    fn inner(&self) -> &Self::RawWindow {
        &self.window
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
