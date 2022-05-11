//! Errors which can happen in various parts of the library.

use lyon::tessellation::TessellationError;
use std::sync::mpsc::SendError;

/// Enumeration of errors which can happen during the operation of the library.
#[derive(Debug)]
pub enum Error {
    Schedule,
    Network(String),
    Tesselation(TessellationError),
}

impl From<TessellationError> for Error {
    fn from(e: TessellationError) -> Self {
        Error::Tesselation(e)
    }
}

impl<T> From<SendError<T>> for Error {
    fn from(_e: SendError<T>) -> Self {
        Error::Schedule
    }
}
