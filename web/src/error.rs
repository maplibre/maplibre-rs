//! Errors from JS world.

use std::{
    borrow::Cow,
    error::Error,
    fmt::{Display, Formatter},
};

use js_sys::{Error as JSError, TypeError};
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
    /// Any other Error
    #[error("Error from JS")]
    GenericError(Cow<'static, str>),
}

impl From<JsValue> for WebError {
    fn from(value: JsValue) -> Self {
        if let Some(error) = value.dyn_ref::<TypeError>() {
            let Some(message) = error
                .message()
                .as_string() else { return WebError::InvalidMessage; };

            WebError::TypeError(message.into())
        } else if let Some(error) = value.dyn_ref::<JSError>() {
            let Some(message) = error
                .message()
                .as_string() else { return WebError::InvalidMessage; };

            WebError::GenericError(message.into())
        } else {
            WebError::UnknownErrorType
        }
    }
}

/// Wraps several unrelated errors and implements Into<JSValue>. This should be used in Rust
/// functions called from JS-land as return error type.
#[derive(Debug)]
pub enum WrappedError {
    ProcedureError(ProcedureError),
    CallError(CallError),
    WebError(WebError),
}

impl Display for WrappedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error from Rust: {:?}", self)
    }
}

impl Error for WrappedError {}

impl From<WrappedError> for JsValue {
    fn from(val: WrappedError) -> Self {
        JsValue::from_str(&val.to_string())
    }
}

impl From<CallError> for WrappedError {
    fn from(e: CallError) -> Self {
        WrappedError::CallError(e)
    }
}

impl From<ProcedureError> for WrappedError {
    fn from(e: ProcedureError) -> Self {
        WrappedError::ProcedureError(e)
    }
}

impl From<WebError> for WrappedError {
    fn from(e: WebError) -> Self {
        WrappedError::WebError(e)
    }
}
