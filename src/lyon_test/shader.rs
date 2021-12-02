use wgpu::{ColorTargetState, FragmentState, ShaderModule, ShaderModuleDescriptor, VertexAttribute, VertexBufferLayout, VertexState};

use crate::shader_ffi::GpuVertex;

const MAP_VERTEX_SHADER_ARGUMENTS: [VertexAttribute; 3] = [
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

const MAP_VERTEX_SHADER_BUFFERS: [VertexBufferLayout; 1] = [wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<GpuVertex>() as u64,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &MAP_VERTEX_SHADER_ARGUMENTS,
}];

const MAP_VERTEX_COLOR_TARGETS: [ColorTargetState; 1] = [wgpu::ColorTargetState {
    format: wgpu::TextureFormat::Bgra8UnormSrgb,
    blend: None,
    write_mask: wgpu::ColorWrites::ALL,
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

pub fn create_map_vertex_state(vertex_shader_module: &ShaderModule) -> VertexState {
    wgpu::VertexState {
        module: vertex_shader_module,
        entry_point: "main",
        buffers: &MAP_VERTEX_SHADER_BUFFERS,
    }
}

pub fn create_map_fragment_state(fragment_shader_module: &ShaderModule) -> FragmentState {
    wgpu::FragmentState {
        module: fragment_shader_module,
        entry_point: "main",
        targets: &MAP_VERTEX_COLOR_TARGETS,
    }
}
