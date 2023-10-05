use thiserror::Error;

use crate::render::graph::RenderGraphError;

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("error in surface")]
    Surface(#[from] wgpu::SurfaceError),
    #[error("error during surface creation")]
    CreateSurfaceError(#[from] wgpu::CreateSurfaceError),
    #[error("error in render graph")]
    Graph(#[from] RenderGraphError),
    #[error("error while requesting device")]
    RequestDevice(#[from] wgpu::RequestDeviceError),
}

impl RenderError {
    pub fn should_exit(&self) -> bool {
        matches!(self, RenderError::Surface(wgpu::SurfaceError::OutOfMemory))
    }
}
