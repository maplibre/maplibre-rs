use log::info;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::MessageEvent;
use web_sys::Window;

#[wasm_bindgen]
pub fn test_fetch(web_window: &Window) {
    let cb: Closure<dyn FnMut(JsValue) + 'static> = Closure::wrap(Box::new(|value: JsValue| {
        info!("interval elapsed!");
    })
        as Box<dyn FnMut(JsValue)>);
    web_window
        .fetch_with_str("http://localhost:5555/web/index.html")
        .then(&cb);

    cb.forget();
}

#[wasm_bindgen]
pub fn test_shared_mem(memory: &JsValue) {
    let worker = web_sys::Worker::new_with_options(
        "./fetch-worker.js",
        // Works only on chrome
        &web_sys::WorkerOptions::new().type_(web_sys::WorkerType::Module),
    )
    .unwrap();
    let callback = Closure::wrap(Box::new(move |event: MessageEvent| {
        info!("{}{:?}", "Received response: ", &event.data());
    }) as Box<dyn FnMut(_)>);

    let sab = js_sys::SharedArrayBuffer::new(10);
    let u8sab = js_sys::Uint8Array::new(sab.as_ref());
    u8sab.set_index(0, 13);

    //worker_handle.set_onmessage(Some(callback.as_ref().unchecked_ref()));
    //worker_handle.post_message(&JsValue::from("hello"));

    info!("test");
    info!("{:?}", &memory);

    /*  let msg = js_sys::Array::new();
      msg.push(memory.as_ref());
      msg.push(&JsValue::from(test_alloc(100)));
      msg.push(&u8sab);
      worker.post_message(&msg.as_ref());
    */

    //callback.forget();
}

#[wasm_bindgen]
pub fn test_alloc() -> *mut u8 {
    let mut buf: Vec<u8> = Vec::with_capacity(100);

    buf.push(56);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    return ptr;
}

#[wasm_bindgen]
pub fn get54(ptr: *mut u8) -> u8 {
    unsafe {
        let data: Vec<u8> = Vec::from_raw_parts(ptr, 100, 100);
        data[0]
    }
}
