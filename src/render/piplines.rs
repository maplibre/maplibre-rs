use wgpu::{FragmentState, PipelineLayout, RenderPipelineDescriptor, VertexState};

use super::texture::DEPTH_TEXTURE_FORMAT;

///
/// Creates a render pipeline description
///
/// # Arguments
///
/// * `pipeline_layout`:
/// * `vertex_state`:
/// * `fragment_state`:
/// * `sample_count`:
/// * `update_stencil`: Fragments passing through the pipeline will be able to update the stencil
///                     buffer. This is used for masking
///
/// returns: RenderPipelineDescriptor
///
/// # Examples
///
/// ```
///
/// ```
pub fn create_map_render_pipeline_description<'a>(
    pipeline_layout: &'a PipelineLayout,
    vertex_state: VertexState<'a>,
    fragment_state: FragmentState<'a>,
    sample_count: u32,
    update_stencil: bool,
) -> RenderPipelineDescriptor<'a> {
    let stencil_state = if update_stencil {
        wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::Always, // Allow ALL values to update the stencil
            fail_op: wgpu::StencilOperation::Keep,
            depth_fail_op: wgpu::StencilOperation::Keep, // This is used when the depth test already failed
            pass_op: wgpu::StencilOperation::Replace,
        }
    } else {
        wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::Equal,
            fail_op: wgpu::StencilOperation::Keep,
            depth_fail_op: wgpu::StencilOperation::Keep,
            pass_op: wgpu::StencilOperation::Keep,
        }
    };
    let descriptor = wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: vertex_state,
        fragment: Some(fragment_state),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            polygon_mode: wgpu::PolygonMode::Fill,
            front_face: wgpu::FrontFace::Ccw,
            strip_index_format: None,
            cull_mode: None, // TODO Maps look the same from he bottom and above
            conservative: false,
            unclipped_depth: false
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_TEXTURE_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Always, // FIXME
            stencil: wgpu::StencilState {
                front: stencil_state,
                back: stencil_state,
                read_mask: 0xff, // Applied to stencil values being read from the stencil buffer
                write_mask: 0xff, // Applied to fragment stencil values before being written to  the stencil buffer
            },
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: sample_count,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None
    };
    descriptor
}
