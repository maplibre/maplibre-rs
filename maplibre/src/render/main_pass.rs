//! The main render pass for this application.
//!
//! Right now there is only one render graph. A use case for multiple render passes would be
//! [shadows](https://www.raywenderlich.com/books/metal-by-tutorials/v2.0/chapters/14-multipass-deferred-rendering).

use std::ops::Deref;

use crate::{
    ecs::world::World,
    render::{
        draw_graph,
        graph::{Node, NodeRunError, RenderContext, RenderGraphContext, SlotInfo},
        render_phase::{Draw, LayerItem, RenderCommand, RenderPhase, TileMaskItem},
        resource::TrackedRenderPass,
        Eventually::Initialized,
        RenderState,
    },
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
        world: &World,
    ) -> Result<(), NodeRunError> {
        let Initialized(render_target) = &state.render_target else {
            return Ok(());
        };
        let Initialized(multisampling_texture) = &state.multisampling_texture else {
            return Ok(());
        };
        let Initialized(depth_texture) = &state.depth_texture else {
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
                    label: Some("main_pass"),
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

        // FIXME: Debug vs tile mask phase?
        let mask_items = world.resources.get::<RenderPhase<TileMaskItem>>().unwrap();
        for item in &mask_items.items {
            item.draw_function
                .draw(&mut tracked_pass, state, world, item);
        }

        let layer_items = world.resources.get::<RenderPhase<LayerItem>>().unwrap();
        // TODO print layer items count
        for item in &layer_items.items {
            item.draw_function
                .draw(&mut tracked_pass, state, world, item);
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
        world: &World,
    ) -> Result<(), NodeRunError> {
        graph.run_sub_graph(draw_graph::NAME, vec![])?;

        Ok(())
    }
}
