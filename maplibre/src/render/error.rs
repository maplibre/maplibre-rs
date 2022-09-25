use std::fmt;

use crate::{error::Error, render::graph::RenderGraphError};

#[derive(Debug)]
pub enum RenderError {
    Surface(wgpu::SurfaceError),
    Graph(RenderGraphError),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::Surface(e) => write!(f, "{}", e),
            RenderError::Graph(e) => write!(f, "{}", e),
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
            _ => true,
        }
    }
}

impl From<wgpu::SurfaceError> for Error {
    fn from(e: wgpu::SurfaceError) -> Self {
        Error::Render(RenderError::Surface(e))
    }
}
