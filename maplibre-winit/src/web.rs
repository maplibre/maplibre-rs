use std::marker::PhantomData;

use maplibre::window::{MapWindow, MapWindowConfig, PhysicalSize, WindowCreateError};
use winit::{platform::web::WindowAttributesExtWebSys, window::WindowAttributes};

use super::WinitMapWindow;
use crate::WinitEventLoop;

#[derive(Clone)]
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

impl<ET: 'static + Clone> MapWindowConfig for WinitMapWindowConfig<ET> {
    type MapWindow = WinitMapWindow<ET>;

    fn create(&self) -> Result<Self::MapWindow, WindowCreateError> {
        let raw_event_loop = winit::event_loop::EventLoop::<ET>::with_user_event()
            .build()
            .map_err(|_| WindowCreateError::EventLoop)?;

        let window: winit::window::Window = raw_event_loop
            .create_window(
                WindowAttributes::default().with_canvas(Some(get_canvas(&self.canvas_id))),
            )
            .map_err(|_| WindowCreateError::Window)?;

        Ok(Self::MapWindow {
            window,
            event_loop: Some(WinitEventLoop {
                event_loop: raw_event_loop,
            }),
        })
    }
}

impl<ET: 'static> MapWindow for WinitMapWindow<ET> {
    fn size(&self) -> PhysicalSize {
        let size = self.window.inner_size();

        PhysicalSize::new(size.width, size.height).unwrap_or(PhysicalSize::new(1, 1).unwrap())
    }
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
