//! Errors which can happen in various parts of the library.

use std::{fmt, fmt::Formatter, sync::mpsc::SendError};

use lyon::tessellation::TessellationError;

use crate::render::{error::RenderError, graph::RenderGraphError};

/// Enumeration of errors which can happen during the operation of the library.
#[derive(Debug)]
pub enum Error {
    Scheduler,
    Network(String),
    Tesselation(TessellationError),
    Render(RenderError),
}

impl From<TessellationError> for Error {
    fn from(e: TessellationError) -> Self {
        Error::Tesselation(e)
    }
}

impl<T> From<SendError<T>> for Error {
    fn from(_e: SendError<T>) -> Self {
        Error::Scheduler
    }
}
