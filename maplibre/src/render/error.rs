use std::fmt;

use crate::{error::Error, render::graph::RenderGraphError};

#[derive(Debug)]
pub enum RenderError {
    Surface(wgpu::SurfaceError),
    Graph(RenderGraphError),
    Device(wgpu::RequestDeviceError),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::Surface(e) => write!(f, "{}", e),
            RenderError::Graph(e) => write!(f, "{:?}", e),
            RenderError::Device(e) => write!(f, "{}", e),
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

impl From<RenderGraphError> for RenderError {
    fn from(e: RenderGraphError) -> Self {
        RenderError::Graph(e)
    }
}

impl From<wgpu::SurfaceError> for Error {
    fn from(e: wgpu::SurfaceError) -> Self {
        Error::Render(RenderError::Surface(e))
    }
}

impl From<wgpu::RequestDeviceError> for Error {
    fn from(e: wgpu::RequestDeviceError) -> Self {
        Error::Render(RenderError::Device(e))
    }
}
