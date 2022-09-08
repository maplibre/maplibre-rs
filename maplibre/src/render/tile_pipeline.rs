//! Utility for declaring pipelines.

use std::cmp;

use crate::{
    platform::MIN_BUFFER_SIZE,
    render::{
        resource::{FragmentState, RenderPipeline, RenderPipelineDescriptor, VertexState},
        settings::Msaa,
        shaders::ShaderGlobals,
    },
};

pub struct TilePipeline {
    bind_globals: bool,
    update_stencil: bool,
    debug_stencil: bool,
    wireframe: bool,
    msaa: Msaa,

    vertex_state: VertexState,
    fragment_state: FragmentState,
}

impl TilePipeline {
    pub(crate) fn new(
        msaa: Msaa,
        vertex_state: VertexState,
        fragment_state: FragmentState,
        bind_globals: bool,
        update_stencil: bool,
        debug_stencil: bool,
        wireframe: bool,
    ) -> Self {
        TilePipeline {
            bind_globals,
            update_stencil,
            debug_stencil,
            wireframe,
            msaa,
            vertex_state,
            fragment_state,
        }
    }
}

impl RenderPipeline for TilePipeline {
    fn describe_render_pipeline(self) -> RenderPipelineDescriptor {
        let stencil_state = if self.update_stencil {
            wgpu::StencilFaceState {
                compare: wgpu::CompareFunction::Always, // Allow ALL values to update the stencil
                fail_op: wgpu::StencilOperation::Keep,
                depth_fail_op: wgpu::StencilOperation::Keep, // This is used when the depth test already failed
                pass_op: wgpu::StencilOperation::Replace,
            }
        } else {
            wgpu::StencilFaceState {
                compare: if self.debug_stencil {
                    wgpu::CompareFunction::Always
                } else {
                    wgpu::CompareFunction::Equal
                },
                fail_op: wgpu::StencilOperation::Keep,
                depth_fail_op: wgpu::StencilOperation::Keep,
                pass_op: wgpu::StencilOperation::Keep,
            }
        };

        let globals_buffer_byte_size =
            cmp::max(MIN_BUFFER_SIZE, std::mem::size_of::<ShaderGlobals>() as u64);

        RenderPipelineDescriptor {
            label: None,
            layout: if self.bind_globals {
                Some(vec![vec![wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(globals_buffer_byte_size),
                    },
                    count: None,
                }]])
            } else {
                None
            },
            vertex: self.vertex_state,
            fragment: self.fragment_state,
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                polygon_mode: if self.update_stencil {
                    wgpu::PolygonMode::Fill
                } else if self.wireframe {
                    wgpu::PolygonMode::Line
                } else {
                    wgpu::PolygonMode::Fill
                },
                front_face: wgpu::FrontFace::Ccw,
                strip_index_format: None,
                cull_mode: None, // Maps look the same from he bottom and above -> No culling needed
                conservative: false,
                unclipped_depth: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: !self.update_stencil,
                depth_compare: wgpu::CompareFunction::Greater,
                stencil: wgpu::StencilState {
                    front: stencil_state,
                    back: stencil_state,
                    read_mask: 0xff, // Applied to stencil values being read from the stencil buffer
                    write_mask: 0xff, // Applied to fragment stencil values before being written to  the stencil buffer
                },
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: self.msaa.samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        }
    }
}
