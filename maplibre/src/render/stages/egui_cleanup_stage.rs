use crate::{
    context::MapContext,
    render::{eventually::Eventually::Initialized, RenderState, Renderer},
    schedule::Stage,
};

#[derive(Default)]
pub struct EguiCleanupStage;

impl Stage for EguiCleanupStage {
    #[tracing::instrument(name = "EguiCleanupStage", skip_all)]
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
        let Initialized(egui_renderer) = egui_renderer else { return; };

        for egui_texture in egui_textures.into_iter() {
            egui_renderer.free_texture(egui_texture);
        }

        egui_textures.clear();
    }
}
