extern crate console_error_panic_hook;

use std::panic;

use log::{warn, Level};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsCast;
use web_sys::Window as WebWindow;
use winit::dpi::{LogicalSize, Size};
use winit::event_loop::EventLoop;
use winit::platform::web::WindowBuilderExtWebSys;
use winit::window::{Window, WindowBuilder};

#[wasm_bindgen(start)]
pub fn run() {
    console_log::init_with_level(Level::Info).expect("error initializing log");
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    wasm_bindgen_futures::spawn_local(async {
        let event_loop = EventLoop::new();
        let web_window: WebWindow = web_sys::window().unwrap();
        let document = web_window.document().unwrap();
        let body = document.body().unwrap();
        let builder = WindowBuilder::new();
        let canvas: web_sys::HtmlCanvasElement = document
            .get_element_by_id("mapr")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();

        let window: Window = builder
            .with_canvas(Some(canvas))
            .build(&event_loop)
            .unwrap();

        window.set_inner_size(LogicalSize {
            width: body.client_width(),
            height: body.client_height(),
        });

        super::setup(window, event_loop).await;
    });
}
