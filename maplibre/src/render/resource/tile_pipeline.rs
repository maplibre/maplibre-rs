//! Utility for declaring pipelines.

use std::borrow::Cow;

use crate::render::{
    resource::{FragmentState, RenderPipeline, RenderPipelineDescriptor, VertexState},
    settings::RendererSettings,
};

pub struct TilePipeline {
    name: Cow<'static, str>,
    /// Is the depth stencil used?
    depth_stencil_enabled: bool,
    /// This pipeline updates the stenctil
    update_stencil: bool,
    /// Force a write and ignore stencil
    debug_stencil: bool,
    wireframe: bool,
    msaa: bool,
    raster: bool,
    settings: RendererSettings,

    vertex_state: VertexState,
    fragment_state: FragmentState,
}

impl TilePipeline {
    pub fn new(
        name: Cow<'static, str>,
        settings: RendererSettings,
        vertex_state: VertexState,
        fragment_state: FragmentState,
        depth_stencil_enabled: bool,
        update_stencil: bool,
        debug_stencil: bool,
        wireframe: bool,
        multisampling: bool,
        raster: bool,
    ) -> Self {
        TilePipeline {
            name,
            depth_stencil_enabled,
            update_stencil,
            debug_stencil,
            wireframe,
            msaa: multisampling,
            raster,
            settings,
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

        RenderPipelineDescriptor {
            label: Some(self.name),
            layout: if self.raster {
                Some(vec![vec![
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ]])
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
            depth_stencil: if !self.depth_stencil_enabled {
                None
            } else {
                Some(wgpu::DepthStencilState {
                    format: self.settings.depth_texture_format,
                    depth_write_enabled: !self.update_stencil,
                    depth_compare: wgpu::CompareFunction::Greater,
                    stencil: wgpu::StencilState {
                        front: stencil_state,
                        back: stencil_state,
                        read_mask: 0xff, // Applied to stencil values being read from the stencil buffer
                        write_mask: 0xff, // Applied to fragment stencil values before being written to  the stencil buffer
                    },
                    bias: wgpu::DepthBiasState::default(),
                })
            },
            multisample: wgpu::MultisampleState {
                count: if self.msaa {
                    self.settings.msaa.samples
                } else {
                    1
                },
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        }
    }
}
