//! Utilities which holds references to GPU-owned. Usually a resource is a wrapper which makes using
//! buffers or textures simpler.

mod buffer_pool;
mod globals;
mod pipeline;
mod raster;
mod shader;
mod surface;
mod texture;
mod tracked_render_pass;

pub use buffer_pool::*;
pub use globals::*;
pub use pipeline::*;
pub use raster::*;
pub use shader::*;
pub use surface::*;
pub use texture::*;
pub use tracked_render_pass::*;

pub trait Queue<B> {
    fn write_buffer(&self, buffer: &B, offset: wgpu::BufferAddress, data: &[u8]);
    fn write_texture(
        &self,
        texture: wgpu::ImageCopyTexture,
        data: &[u8],
        data_layout: wgpu::ImageDataLayout,
        size: wgpu::Extent3d,
    );
}

impl Queue<wgpu::Buffer> for wgpu::Queue {
    fn write_buffer(&self, buffer: &wgpu::Buffer, offset: wgpu::BufferAddress, data: &[u8]) {
        self.write_buffer(buffer, offset, data)
    }
    fn write_texture(
        &self,
        texture: wgpu::ImageCopyTexture,
        data: &[u8],
        data_layout: wgpu::ImageDataLayout,
        size: wgpu::Extent3d,
    ) {
        self.write_texture(texture, data, data_layout, size)
    }
}
