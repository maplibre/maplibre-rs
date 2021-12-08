use log::info;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
use web_sys::Window;

pub fn test_fetch(web_window: &Window) {
    let cb: Closure<dyn FnMut(JsValue) + 'static> = Closure::wrap(Box::new(|value: JsValue| {
        info!("interval elapsed!");
    }) as Box<dyn FnMut(JsValue)>);
    web_window.fetch_with_str("http://localhost:5555/web/mapr.html").then(&cb);

    cb.forget();
}
