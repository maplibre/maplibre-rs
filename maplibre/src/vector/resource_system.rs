//! Prepares GPU-owned resources by initializing them if they are uninitialized or out-of-date.

use std::mem::size_of;

use crate::{
    context::MapContext,
    render::{
        eventually::Eventually,
        resource::{BackingBufferDescriptor, RenderPipeline, TilePipeline},
        shaders,
        shaders::{Shader, ShaderTileMetadata},
        tile_view_pattern::{TileViewPattern, WgpuTileViewPattern, DEFAULT_TILE_VIEW_PATTERN_SIZE},
        RenderResources, Renderer,
    },
    vector::{resource::BufferPool, MaskPipeline, VectorBufferPool, VectorPipeline},
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
    let Some((
        buffer_pool,
        tile_view_pattern,
        vector_pipeline,
        mask_pipeline
    )) = world.resources.query_mut::<(
        &mut Eventually<VectorBufferPool>,
        &mut Eventually<WgpuTileViewPattern>,
        &mut Eventually<VectorPipeline>,
        &mut Eventually<MaskPipeline>,
    )>() else { return; };

    buffer_pool.initialize(|| BufferPool::from_device(device));

    tile_view_pattern.initialize(|| {
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

    vector_pipeline.initialize(|| {
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

    mask_pipeline.initialize(|| {
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
}
