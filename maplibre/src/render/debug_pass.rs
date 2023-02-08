use std::ops::Deref;

use crate::{
    ecs::world::World,
    render::{
        graph::{Node, NodeRunError, RenderContext, RenderGraphContext, SlotInfo},
        render_phase::{Draw, RenderCommand, RenderPhase, TileMaskItem},
        resource::TrackedRenderPass,
        Eventually::Initialized,
        RenderState,
    },
};

/// Pass which renders debug information on top of the map.
pub struct DebugPassNode {}

impl DebugPassNode {
    pub fn new() -> Self {
        Self {}
    }
}

impl Node for DebugPassNode {
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

        let color_attachment = wgpu::RenderPassColorAttachment {
            view: render_target.deref(),
            ops: wgpu::Operations {
                // Draws on-top of previously rendered data
                load: wgpu::LoadOp::Load,
                store: true,
            },
            resolve_target: None,
        };

        let render_pass =
            render_context
                .command_encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("debug_pass"),
                    color_attachments: &[Some(color_attachment)],
                    depth_stencil_attachment: None,
                });

        let mut tracked_pass = TrackedRenderPass::new(render_pass);

        // FIXME tcs: Debug vs tile mask phase?
        /*for item in &world
            .get_resource::<RenderPhase<TileMaskItem>>()
            .items
            .clone()
        {
            let draw_function = draw_functions.get(item.draw_function).unwrap();
            draw_function.draw(&mut tracked_pass, state, world, item);
        }*/

        Ok(())
    }
}
