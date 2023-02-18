//! Prepares GPU-owned resources by initializing them if they are uninitialized or out-of-date.
use crate::{
    context::MapContext,
    raster::resource::RasterResources,
    render::{
        eventually::Eventually,
        resource::{RenderPipeline, TilePipeline},
        settings::Msaa,
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
    let Some(raster_resources) = world
        .resources
        .query_mut::<&mut Eventually<RasterResources>>() else { return; };

    raster_resources.initialize(|| {
        let shader = shaders::RasterTileShader {
            format: surface.surface_format(),
        };

        RasterResources::new(
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
                surface.is_multisampling_supported(settings.msaa),
                true,
            )
            .describe_render_pipeline()
            .initialize(device),
        )
    });
}
