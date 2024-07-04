//! Errors from JS world.

use std::{
    borrow::Cow,
    fmt::{Display, Formatter},
};

use js_sys::TypeError;
use maplibre::io::apc::{CallError, ProcedureError};
use thiserror::Error;
use wasm_bindgen::{JsCast, JsValue};

#[derive(Error, Debug)]
pub enum WebError {
    #[error("JS error type is unknown")]
    UnknownErrorType,
    /// Returned if the message is not valid, e.g. if it it is not valid UTF-8.
    #[error("message string in error is invalid")]
    InvalidMessage,
    /// TypeError like it is defined in JS
    #[error("TypeError from JS")]
    TypeError(Cow<'static, str>),
    #[error("fetching data failed")]
    FetchError(Cow<'static, str>),
    /// Any other Error
    #[error("Error from JS")]
    GenericError(Cow<'static, str>),
}

impl From<JsValue> for WebError {
    fn from(value: JsValue) -> Self {
        if let Some(error) = value.dyn_ref::<TypeError>() {
            let Some(message) = error.message().as_string() else {
                return WebError::InvalidMessage;
            };

            WebError::TypeError(message.into())
        } else if let Some(error) = value.dyn_ref::<js_sys::Error>() {
            let Some(message) = error.message().as_string() else {
                return WebError::InvalidMessage;
            };

            WebError::GenericError(message.into())
        } else {
            WebError::UnknownErrorType
        }
    }
}

/// Wraps several unrelated errors and implements Into<JSValue>. This should be used in Rust
/// functions called from JS-land as return error type.
#[derive(Error, Debug)]
pub enum JSError {
    Procedure(#[from] ProcedureError),
    Call(#[from] CallError),
    Web(#[from] WebError),
}

impl Display for JSError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            JSError::Procedure(inner) => inner.fmt(f),
            JSError::Call(inner) => inner.fmt(f),
            JSError::Web(inner) => inner.fmt(f),
        }
    }
}

impl From<JSError> for JsValue {
    fn from(val: JSError) -> Self {
        JsValue::from_str(&val.to_string())
    }
}
