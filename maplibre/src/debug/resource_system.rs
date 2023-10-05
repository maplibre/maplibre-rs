//! Prepares GPU-owned resources by initializing them if they are uninitialized or out-of-date.
use crate::{
    context::MapContext,
    debug::DebugPipeline,
    render::{
        eventually::Eventually,
        resource::{RenderPipeline, TilePipeline},
        shaders,
        shaders::Shader,
        RenderResources, Renderer,
    },
};

pub fn resource_system(
    MapContext {
        world,
        renderer:
            Renderer {
                device,
                resources: RenderResources { surface, .. },
                settings,
                ..
            },
        ..
    }: &mut MapContext,
) {
    let Some(debug_pipeline) = world
        .resources
        .query_mut::<&mut Eventually<DebugPipeline>>()
    else {
        return;
    };

    debug_pipeline.initialize(|| {
        let mask_shader = shaders::TileMaskShader {
            format: surface.surface_format(),
            draw_colors: true,
            debug_lines: true,
        };

        let pipeline = TilePipeline::new(
            "debug_pipeline".into(),
            *settings,
            mask_shader.describe_vertex(),
            mask_shader.describe_fragment(),
            false,
            false,
            true,
            false,
            false,
            false,
        )
        .describe_render_pipeline()
        .initialize(device);
        DebugPipeline(pipeline)
    });
}
