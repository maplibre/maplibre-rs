//! The main render pass for this application.
//!
//! Right now there is only one render graph. A use case for multiple render passes would be
//! [shadows](https://www.raywenderlich.com/books/metal-by-tutorials/v2.0/chapters/14-multipass-deferred-rendering).

use std::ops::Deref;

use wgpu::StoreOp;

use crate::{
    render::{
        draw_graph,
        graph::{Node, NodeRunError, RenderContext, RenderGraphContext, SlotInfo},
        render_phase::{LayerItem, RenderPhase, TileMaskItem},
        resource::TrackedRenderPass,
        Eventually::Initialized,
        RenderResources,
    },
    tcs::world::World,
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

    fn update(&mut self, _state: &mut RenderResources) {}

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        state: &RenderResources,
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
                    store: StoreOp::Store,
                },
                resolve_target: Some(render_target.deref()),
            }
        } else {
            wgpu::RenderPassColorAttachment {
                view: render_target.deref(),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: StoreOp::Store,
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
                            store: StoreOp::Store,
                        }),
                        stencil_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(0),
                            store: StoreOp::Store,
                        }),
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

        let mut tracked_pass = TrackedRenderPass::new(render_pass);

        if let Some(mask_items) = world.resources.get::<RenderPhase<TileMaskItem>>() {
            log::trace!("RenderPhase<TileMaskItem>::size() = {}", mask_items.size());
            for item in mask_items {
                item.draw_function.draw(&mut tracked_pass, world, item);
            }
        }

        if let Some(layer_items) = world.resources.get::<RenderPhase<LayerItem>>() {
            log::trace!("RenderPhase<LayerItem>::size() = {}", layer_items.size());
            for item in layer_items {
                item.draw_function.draw(&mut tracked_pass, world, item);
            }
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
        _resources: &RenderResources,
        _world: &World,
    ) -> Result<(), NodeRunError> {
        graph.run_sub_graph(draw_graph::NAME, vec![])?;

        Ok(())
    }
}
