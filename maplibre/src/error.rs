//! Errors which can happen in various parts of the library.

use std::{fmt, fmt::Formatter, sync::mpsc::SendError};

use lyon::tessellation::TessellationError;

#[derive(Debug)]
pub enum RenderError {
    Surface(wgpu::SurfaceError),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::Surface(e) => write!(f, "{}", e),
        }
    }
}

impl RenderError {
    pub fn should_exit(&self) -> bool {
        match self {
            RenderError::Surface(e) => match e {
                wgpu::SurfaceError::OutOfMemory => true,
                _ => false,
            },
        }
    }
}

/// Enumeration of errors which can happen during the operation of the library.
#[derive(Debug)]
pub enum Error {
    Schedule,
    Network(String),
    Tesselation(TessellationError),
    Render(RenderError),
}

impl From<wgpu::SurfaceError> for Error {
    fn from(e: wgpu::SurfaceError) -> Self {
        Error::Render(RenderError::Surface(e))
    }
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
