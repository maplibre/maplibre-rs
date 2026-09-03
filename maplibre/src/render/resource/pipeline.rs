//! Utility for creating [RenderPipelines](wgpu::RenderPipeline)

use std::borrow::Cow;

use crate::render::resource::shader::{FragmentState, VertexState};

pub trait RenderPipeline {
    fn describe_render_pipeline(self) -> RenderPipelineDescriptor;
}

pub struct RenderPipelineDescriptor {
    /// Debug label of the pipeline. This will show up in graphics debuggers for easy identification.
    pub label: Option<Cow<'static, str>>,
    /// The layout of bind groups for this pipeline.
    pub layout: Option<Vec<Vec<wgpu::BindGroupLayoutEntry>>>,
    /// The compiled vertex stage, its entry point, and the input buffers layout.
    pub vertex: VertexState,
    /// The properties of the pipeline at the primitive assembly and rasterization level.
    pub primitive: wgpu::PrimitiveState,
    /// The effect of draw calls on the depth and stencil aspects of the output target, if any.
    pub depth_stencil: Option<wgpu::DepthStencilState>,
    /// The multi-sampling properties of the pipeline.
    pub multisample: wgpu::MultisampleState,
    /// The compiled fragment stage, its entry point, and the color targets.
    pub fragment: FragmentState,
}

impl RenderPipelineDescriptor {
    pub fn initialize(&self, device: &wgpu::Device) -> wgpu::RenderPipeline {
        self.initialize_with_prefix_layouts(device, &[])
    }

    /// Creates a pipeline with shared bind-group layouts before descriptor-owned layouts.
    pub fn initialize_with_prefix_layouts(
        &self,
        device: &wgpu::Device,
        prefix_layouts: &[&wgpu::BindGroupLayout],
    ) -> wgpu::RenderPipeline {
        let bind_group_layouts = if let Some(layout) = &self.layout {
            layout
                .iter()
                .map(|entries| {
                    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: None,
                        entries: entries.as_ref(),
                    })
                })
                .collect::<Vec<_>>()
        } else {
            vec![]
        };

        let all_bind_group_layouts = prefix_layouts
            .iter()
            .copied()
            .chain(bind_group_layouts.iter())
            .collect::<Vec<_>>();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &all_bind_group_layouts,
            ..Default::default()
        });

        let vertex_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(self.vertex.source.into()),
        });
        let fragment_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(self.fragment.source.into()),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: self.label.as_ref().map(|label| label.as_ref()),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader_module,
                entry_point: self.vertex.entry_point,
                compilation_options: Default::default(),
                buffers: self
                    .vertex
                    .buffers
                    .iter()
                    .map(|layout| wgpu::VertexBufferLayout {
                        array_stride: layout.array_stride,
                        step_mode: layout.step_mode,
                        attributes: layout.attributes.as_slice(),
                    })
                    .collect::<Vec<_>>()
                    .as_slice(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader_module,
                entry_point: self.fragment.entry_point,
                compilation_options: Default::default(),
                targets: self.fragment.targets.as_slice(),
            }),
            primitive: self.primitive,
            depth_stencil: self.depth_stencil.clone(),
            multisample: self.multisample,

            multiview: None,
            cache: None,
        });

        pipeline
    }
}
