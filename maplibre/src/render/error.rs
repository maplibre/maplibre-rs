use thiserror::Error;

use crate::render::graph::RenderGraphError;

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("error in surface")]
    Surface(#[from] wgpu::SurfaceError),
    #[error("error while getting window handle")]
    Handle(#[from] wgpu::rwh::HandleError),
    #[error("error during surface creation")]
    CreateSurfaceError(#[from] wgpu::CreateSurfaceError),
    #[error("error in render graph")]
    Graph(#[from] RenderGraphError),
    #[error("error while requesting device")]
    RequestDevice(#[from] wgpu::RequestDeviceError),
    #[error("error while requesting adaptor")]
    RequestAdaptor,
}

impl RenderError {
    pub fn should_exit(&self) -> bool {
        matches!(self, RenderError::Surface(wgpu::SurfaceError::OutOfMemory))
    }
}
