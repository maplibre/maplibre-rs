use egui::RawInput;

use crate::{
    context::MapContext,
    render::{eventually::Eventually::Initialized, RenderState, Renderer},
    schedule::Stage,
};

#[derive(Default)]
pub struct EguiStage;

impl Stage for EguiStage {
    #[tracing::instrument(name = "EguiStage", skip_all)]
    fn run(
        &mut self,
        MapContext {
            renderer:
                Renderer {
                    device,
                    queue,
                    state:
                        RenderState {
                            surface,
                            mask_phase,
                            tile_phase,
                            tile_view_pattern,
                            buffer_pool,
                            egui_renderer,
                            egui_app,
                            egui_context,
                            egui_paint_jobs,
                            egui_textures,
                            ..
                        },
                    ..
                },
            ..
        }: &mut MapContext,
    ) {
        egui_renderer
            .initialize(|| egui_wgpu::Renderer::new(&device, surface.surface_format(), None, 1));

        let Initialized(egui_renderer) = egui_renderer else { return; };

        // Draw the demo application.
        let full_output = egui_context.run(RawInput::default(), |context| {
            egui_app.ui(context);
        });

        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: [surface.size().width(), surface.size().height()],
            pixels_per_point: 1.0,
        };

        let paint_jobs = egui_context.tessellate(full_output.shapes);

        {
            egui_textures.clear();
            for (id, image_delta) in &full_output.textures_delta.set {
                egui_renderer.update_texture(&device, &queue, *id, image_delta);
            }

            for id in &full_output.textures_delta.free {
                egui_textures.push(*id);
            }
        }

        {
            let mut command_encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

            egui_renderer.update_buffers(
                &device,
                &queue,
                &mut command_encoder,
                &paint_jobs,
                &screen_descriptor,
            );

            queue.submit(vec![command_encoder.finish()]);

            egui_paint_jobs.clear();
            egui_paint_jobs.extend(paint_jobs);
        }
    }
}
