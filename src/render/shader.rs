use wgpu::{
    ColorTargetState, FragmentState, ShaderModule, ShaderModuleDescriptor, VertexAttribute,
    VertexBufferLayout, VertexState,
};

use super::platform_constants::COLOR_TEXTURE_FORMAT;
use super::shader_ffi::GpuVertexUniform;

const VERTEX_SHADER_ARGUMENTS: [VertexAttribute; 3] = [
    wgpu::VertexAttribute {
        offset: 0,
        format: wgpu::VertexFormat::Float32x2,
        shader_location: 0,
    },
    wgpu::VertexAttribute {
        offset: wgpu::VertexFormat::Float32x2.size(),
        format: wgpu::VertexFormat::Float32x2,
        shader_location: 1,
    },
    wgpu::VertexAttribute {
        offset: 2 * wgpu::VertexFormat::Float32x2.size(),
        format: wgpu::VertexFormat::Uint32,
        shader_location: 2,
    },
];

const VERTEX_SHADER_BUFFERS: [VertexBufferLayout; 1] = [wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<GpuVertexUniform>() as u64,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &VERTEX_SHADER_ARGUMENTS,
}];

const DEFAULT_FRAGMENT_COLOR_TARGETS: [ColorTargetState; 1] = [wgpu::ColorTargetState {
    format: COLOR_TEXTURE_FORMAT,
    blend: None,
    write_mask: wgpu::ColorWrites::ALL,
}];

const NO_COLOR_FRAGMENT_COLOR_TARGETS: [ColorTargetState; 1] = [wgpu::ColorTargetState {
    format: COLOR_TEXTURE_FORMAT,
    blend: None,
    write_mask: wgpu::ColorWrites::empty(),
}];

pub fn create_vertex_module_descriptor<'a>() -> ShaderModuleDescriptor<'a> {
    wgpu::ShaderModuleDescriptor {
        label: Some("Geometry vs"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/vertex.wgsl").into()),
    }
}

pub fn create_fragment_module_descriptor<'a>() -> ShaderModuleDescriptor<'a> {
    wgpu::ShaderModuleDescriptor {
        label: Some("Geometry fs"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/fragment.wgsl").into()),
    }
}

pub fn create_vertex_state(vertex_shader_module: &ShaderModule) -> VertexState {
    wgpu::VertexState {
        module: vertex_shader_module,
        entry_point: "main",
        buffers: &VERTEX_SHADER_BUFFERS,
    }
}

pub fn create_fragment_state(
    fragment_shader_module: &ShaderModule,
    disable_color: bool,
) -> FragmentState {
    wgpu::FragmentState {
        module: fragment_shader_module,
        entry_point: "main",
        targets: if disable_color {
            &NO_COLOR_FRAGMENT_COLOR_TARGETS
        } else {
            &DEFAULT_FRAGMENT_COLOR_TARGETS
        },
    }
}
