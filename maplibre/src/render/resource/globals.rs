//! A bind group which binds a buffer with global data like the current camera transformations.

use std::{cmp, mem::size_of};

use crate::{platform::MIN_WEBGL_BUFFER_SIZE, render::shaders::ShaderGlobals};

pub struct Globals {
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl Globals {
    pub fn from_device(device: &wgpu::Device, group: &wgpu::BindGroupLayout) -> Self {
        let globals_buffer_byte_size =
            cmp::max(MIN_WEBGL_BUFFER_SIZE, size_of::<ShaderGlobals>() as u64);

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Globals ubo"),
            size: globals_buffer_byte_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind group"),
            layout: group,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(uniform_buffer.as_entire_buffer_binding()),
            }],
        });
        Self {
            uniform_buffer,
            bind_group,
        }
    }
}
