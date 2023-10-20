//! Utilities which holds references to GPU-owned. Usually a resource is a wrapper which makes using
//! buffers or textures simpler.

pub use buffer::*;
pub use pipeline::*;
pub use shader::*;
pub use surface::*;
pub use texture::*;
pub use tile_pipeline::*;
pub use tracked_render_pass::*;

mod buffer;
mod pipeline;
mod shader;
mod surface;
mod texture;
mod tile_pipeline;
mod tracked_render_pass;

pub trait Queue<B> {
    fn write_buffer(&self, buffer: &B, offset: wgpu::BufferAddress, data: &[u8]);
}

impl Queue<wgpu::Buffer> for wgpu::Queue {
    fn write_buffer(&self, buffer: &wgpu::Buffer, offset: wgpu::BufferAddress, data: &[u8]) {
        self.write_buffer(buffer, offset, data)
    }
}
