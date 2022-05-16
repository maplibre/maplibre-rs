use crate::render::graph::{Node, NodeRunError, RenderContext, RenderGraphContext, SlotInfo};
use crate::render::render_commands::{DrawMask, DrawMasks, DrawTile, DrawTiles};
use crate::render::render_phase::{DrawFunctionId, PhaseItem, RenderCommand, TrackedRenderPass};
use crate::render::stages::draw_graph;
use crate::render::util::FloatOrd;
use crate::render::Eventually::Initialized;
use crate::render::RenderState;
use std::ops::{Deref, Range};

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct PipelineID(usize);

pub struct Transparent2d {
    pub sort_key: FloatOrd,
    pub pipeline: PipelineID,
    pub draw_function: DrawFunctionId,
    /// Range in the vertex buffer of this item
    pub batch_range: Option<Range<u32>>,
}

impl PhaseItem for Transparent2d {
    type SortKey = FloatOrd;

    #[inline]
    fn sort_key(&self) -> Self::SortKey {
        self.sort_key
    }

    #[inline]
    fn draw_function(&self) -> DrawFunctionId {
        self.draw_function
    }
}

pub struct MainPassNode {}

impl MainPassNode {
    pub fn new() -> Self {
        Self {}
    }
}

impl Node for MainPassNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![]
    }

    fn update(&mut self, _state: &mut RenderState) {}

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        state: &RenderState,
    ) -> Result<(), NodeRunError> {
        let (render_target, multisampling_texture, depth_texture) = if let (
            Initialized(render_target),
            Initialized(multisampling_texture),
            Initialized(depth_texture),
        ) = (
            &state.render_target,
            &state.multisampling_texture,
            &state.depth_texture,
        ) {
            (render_target, multisampling_texture, depth_texture)
        } else {
            return Ok(());
        };

        let color_attachment = if let Some(texture) = multisampling_texture {
            wgpu::RenderPassColorAttachment {
                view: &texture.view,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: true,
                },
                resolve_target: Some(render_target.deref()),
            }
        } else {
            wgpu::RenderPassColorAttachment {
                view: render_target.deref(),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: true,
                },
                resolve_target: None,
            }
        };

        let render_pass =
            render_context
                .command_encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[color_attachment],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &depth_texture.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(0.0),
                            store: true,
                        }),
                        stencil_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(0),
                            store: true,
                        }),
                    }),
                });

        let mut tracked_pass = TrackedRenderPass::new(render_pass);

        for item in &state.mask_phase.items {
            DrawMasks::render(state, item, &mut tracked_pass);
        }

        for item in &state.tile_phase.items {
            DrawTiles::render(state, item, &mut tracked_pass);
        }
        Ok(())
    }
}

pub struct MainPassDriverNode;

impl Node for MainPassDriverNode {
    fn run(
        &self,
        graph: &mut RenderGraphContext,
        _render_context: &mut RenderContext,
        _state: &RenderState,
    ) -> Result<(), NodeRunError> {
        graph.run_sub_graph(draw_graph::NAME, vec![])?;

        Ok(())
    }
}
