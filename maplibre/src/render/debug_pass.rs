use std::ops::Deref;

use crate::render::{
    graph::{Node, NodeRunError, RenderContext, RenderGraphContext, SlotInfo},
    render_commands::DrawDebugOutlines,
    render_phase::RenderCommand,
    resource::TrackedRenderPass,
    Eventually::Initialized,
    RenderState,
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
        RenderContext {
            command_encoder, ..
        }: &mut RenderContext,
        state: &RenderState,
    ) -> Result<(), NodeRunError> {
        let RenderState {
            surface,
            mask_phase,
            render_target,
            egui_renderer,
            egui_paint_jobs,
            ..
        } = state;

        let Initialized(render_target) = &render_target else {
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

        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(color_attachment.clone())],
                depth_stencil_attachment: None,
            });

            // Debug outlines
            let mut tracked_pass = TrackedRenderPass::new(render_pass);

            for item in &mask_phase.items {
                DrawDebugOutlines::render(state, item, &mut tracked_pass);
            }
        }

        {
            let Initialized(egui_renderer) = egui_renderer else { return Ok(()); };

            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(color_attachment)],
                depth_stencil_attachment: None,
            });

            let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
                size_in_pixels: [surface.size().width(), surface.size().height()],
                pixels_per_point: 1.0,
            };

            // Record all render passes
            egui_renderer.render(&mut render_pass, &egui_paint_jobs, &screen_descriptor);
        }

        Ok(())
    }
}
