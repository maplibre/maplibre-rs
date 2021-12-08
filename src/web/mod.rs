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

#[wasm_bindgen(start)]
pub fn start() {
    console_log::init_with_level(Level::Info).expect("error initializing log");
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    //wasm_bindgen_futures::spawn_local(run());
}

#[wasm_bindgen]
pub async fn run() {
    let worker = web_sys::Worker::new("./fetch-worker.js").unwrap();
    let callback = Closure::wrap(Box::new(move |event: MessageEvent| {
        info!("{}{:?}", "Received response: ", &event.data());
    }) as Box<dyn FnMut(_)>);

    let sab = js_sys::SharedArrayBuffer::new(10);
    let u8sab = js_sys::Uint8Array::new(sab.as_ref());
    u8sab.set_index(0, 13);

    //worker_handle.set_onmessage(Some(callback.as_ref().unchecked_ref()));
    //worker_handle.post_message(&JsValue::from("hello"));

    worker.post_message(&u8sab.as_ref());

    //callback.forget();

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

    super::setup(window, event_loop, Some(u8sab)).await;
}
