//! The main render pass for this application.
//!
//! Right now there is only one render graph. A use case for multiple render passes would be
//! [shadows](https://www.raywenderlich.com/books/metal-by-tutorials/v2.0/chapters/14-multipass-deferred-rendering).

use std::ops::Deref;

use crate::render::{
    draw_graph,
    graph::{Node, NodeRunError, RenderContext, RenderGraphContext, SlotInfo},
    render_commands::{DrawMasks, DrawTiles},
    render_phase::RenderCommand,
    resource::TrackedRenderPass,
    Eventually::Initialized,
    RenderState,
};

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
                    color_attachments: &[Some(color_attachment)],
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
