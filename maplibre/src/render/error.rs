use thiserror::Error;

use crate::render::{graph::RenderGraphError, resource::SurfaceInitError};

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("error while initializing surface")]
    SurfaceInit(#[from] SurfaceInitError),
    #[error("error in surface")]
    Surface(#[from] wgpu::SurfaceError),
    #[error("error in render graph")]
    Graph(#[from] RenderGraphError),
    #[error("error while requesting device")]
    RequestDevice(#[from] wgpu::RequestDeviceError),
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
