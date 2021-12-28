extern crate console_error_panic_hook;

use std::cell::RefCell;
use std::panic;
use std::rc::Rc;

use js_sys::Array;
use log::{info, warn, Level};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, Window as WebWindow};
use winit::dpi::{LogicalSize, Size};
use winit::event_loop::EventLoop;
use winit::platform::web::WindowBuilderExtWebSys;
use winit::window::{Window, WindowBuilder};

mod io;
mod wasm_experiment;

pub use instant::Instant;

// WebGPU
#[cfg(not(feature = "web-webgl"))]
pub const COLOR_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

// WebGL
#[cfg(feature = "web-webgl")]
pub const COLOR_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

#[wasm_bindgen(start)]
pub fn start() {
    if let Err(_) = console_log::init_with_level(Level::Info) {
        // Failed to initialize logging. No need to log a message.
    }
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[wasm_bindgen]
pub async fn run() {
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

    super::setup::setup(window, event_loop).await;
}

