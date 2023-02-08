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
        .get_mut::<Eventually<VectorBufferPool>>()
        .unwrap()
        .initialize(|| BufferPool::from_device(device));

    world
        .resources
        .get_mut::<Eventually<TileViewPattern<wgpu::Queue, wgpu::Buffer>>>()
        .unwrap()
        .initialize(|| {
            let tile_view_buffer_desc = wgpu::BufferDescriptor {
                label: Some("tile view buffer"),
                size: size_of::<ShaderTileMetadata>() as wgpu::BufferAddress
                    * DEFAULT_TILE_VIEW_PATTERN_SIZE,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            };

            TileViewPattern::new(BackingBufferDescriptor::new(
                device.create_buffer(&tile_view_buffer_desc),
                tile_view_buffer_desc.size,
            ))
        });

    world
        .resources
        .get_mut::<Eventually<VectorPipeline>>()
        .unwrap()
        .initialize(|| {
            let tile_shader = shaders::VectorTileShader {
                format: surface.surface_format(),
            };

            let pipeline = TilePipeline::new(
                "vector_pipeline".into(),
                *settings,
                tile_shader.describe_vertex(),
                tile_shader.describe_fragment(),
                true,
                false,
                false,
                false,
                true,
                false,
            )
            .describe_render_pipeline()
            .initialize(device);

            VectorPipeline(pipeline)
        });

    world
        .resources
        .get_mut::<Eventually<MaskPipeline>>()
        .unwrap()
        .initialize(|| {
            let mask_shader = shaders::TileMaskShader {
                format: surface.surface_format(),
                draw_colors: false,
                debug_lines: false,
            };

            let pipeline = TilePipeline::new(
                "mask_pipeline".into(),
                *settings,
                mask_shader.describe_vertex(),
                mask_shader.describe_fragment(),
                true,
                true,
                false,
                false,
                true,
                false,
            )
            .describe_render_pipeline()
            .initialize(device);
            MaskPipeline(pipeline)
        });

    world
        .resources
        .get_mut::<Eventually<DebugPipeline>>()
        .unwrap()
        .initialize(|| {
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
