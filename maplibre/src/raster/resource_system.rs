//! Prepares GPU-owned resources by initializing them if they are uninitialized or out-of-date.
use crate::{
    context::MapContext,
    raster::resource::RasterResources,
    render::{
        eventually::Eventually, resource::RenderPipeline, settings::Msaa, shaders, shaders::Shader,
        tile_pipeline::TilePipeline, RenderState, Renderer,
    },
};

pub fn resource_system(
    MapContext {
        world,
        renderer:
            Renderer {
                device,
                state: RenderState { surface, .. },
                settings,
                ..
            },
        ..
    }: &mut MapContext,
) {
    let Some(raster_resources) = world
        .resources
        .query_mut::<&mut Eventually<RasterResources>>() else { return; };

    raster_resources.initialize(|| {
        let shader = shaders::RasterTileShader {
            format: surface.surface_format(),
        };

        let mut raster_resources = RasterResources::new(
            Msaa { samples: 1 },
            device,
            TilePipeline::new(
                "raster_pipeline".into(),
                *settings,
                shader.describe_vertex(),
                shader.describe_fragment(),
                true,
                false,
                false,
                false,
                true,
                true,
            )
            .describe_render_pipeline()
            .initialize(device),
        );

        raster_resources
    });
}
