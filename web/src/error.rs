//! Errors which can happen in various parts of the library.

use js_sys::Error as JSError;
use wasm_bindgen::{JsCast, JsValue};

#[derive(Debug)]
pub struct WebError(pub String);

impl From<JsValue> for WebError {
    fn from(maybe_error: JsValue) -> Self {
        assert!(maybe_error.is_instance_of::<JSError>());
        let error: JSError = maybe_error.dyn_into().unwrap();
        WebError(error.message().as_string().unwrap())
    }
}
