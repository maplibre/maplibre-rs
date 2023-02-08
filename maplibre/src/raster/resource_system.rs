//! Prepares GPU-owned resources by initializing them if they are uninitialized or out-of-date.

use std::mem::size_of;

use crate::{
    context::MapContext,
    ecs::world::World,
    render::{
        eventually::Eventually,
        resource::{BackingBufferDescriptor, BufferPool, RasterResources, RenderPipeline, Texture},
        settings::Msaa,
        shaders,
        shaders::{Shader, ShaderTileMetadata},
        tile_pipeline::TilePipeline,
        tile_view_pattern::{TileViewPattern, DEFAULT_TILE_VIEW_PATTERN_SIZE},
        RenderState, Renderer,
    },
    vector::{DebugPipeline, MaskPipeline, VectorBufferPool, VectorPipeline},
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
    world
        .resources
        .get_mut::<Eventually<RasterResources>>()
        .unwrap()
        .initialize(|| {
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
