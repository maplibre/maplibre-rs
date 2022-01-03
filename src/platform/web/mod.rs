use std::cell::RefCell;
use std::panic;
use std::rc::Rc;

use log::{info, warn, Level};
use winit::dpi::{LogicalSize, Size};
use winit::event_loop::EventLoop;
use winit::platform::web::WindowBuilderExtWebSys;
use winit::window::{Window, WindowBuilder};

use console_error_panic_hook;
pub use instant::Instant;
use js_sys::Array;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{MessageEvent, Window as WebWindow};

use crate::io::cache::Cache;

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
pub async fn run(cache_ptr: *mut Cache) {
    let cache: Box<Cache> = unsafe { Box::from_raw(cache_ptr) };

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

    // Either call forget or the main loop to keep cache alive
    //std::mem::forget(cache);
    crate::main_loop::setup(window, event_loop, cache).await;
}

#[wasm_bindgen]
pub fn create_cache() -> *mut Cache {
    let mut cache = Box::new(Cache::new());
    let ptr = Box::into_raw(cache);
    return ptr;
}

#[wasm_bindgen]
pub async fn run_cache_loop(cache_ptr: *mut Cache) {
    let mut cache: Box<Cache> = unsafe { Box::from_raw(cache_ptr) };

    // Either call forget or the cache loop to keep cache alive
    cache.run_loop();
    std::mem::forget(cache);
}
