//! Prepares GPU-owned resources by initializing them if they are uninitialized or out-of-date.

use std::mem::size_of;

use crate::{
    context::MapContext,
    render::{
        resource::{BackingBufferDescriptor, BufferPool, Globals, RenderPipeline, Texture},
        shaders,
        shaders::{Shader, ShaderTileMetadata},
        tile_pipeline::TilePipeline,
        tile_view_pattern::{TileViewPattern, DEFAULT_TILE_VIEW_SIZE},
        Renderer,
    },
    schedule::Stage,
};

#[derive(Default)]
pub struct ResourceStage;

impl Stage for ResourceStage {
    #[tracing::instrument(name = "ResourceStage", skip_all)]
    fn run(
        &mut self,
        MapContext {
            renderer:
                Renderer {
                    settings,
                    device,
                    state,
                    ..
                },
            ..
        }: &mut MapContext,
    ) {
        let surface = &mut state.surface;

        let size = surface.size();

        surface.reconfigure(device);

        state
            .render_target
            .initialize(|| surface.create_view(device));

        state.depth_texture.reinitialize(
            || {
                Texture::new(
                    Some("depth texture"),
                    device,
                    settings.depth_texture_format,
                    size.width(),
                    size.height(),
                    settings.msaa,
                )
            },
            &(size.width(), size.height()),
        );

        state.multisampling_texture.reinitialize(
            || {
                if settings.msaa.is_active() {
                    Some(Texture::new(
                        Some("multisampling texture"),
                        device,
                        settings.texture_format,
                        size.width(),
                        size.height(),
                        settings.msaa,
                    ))
                } else {
                    None
                }
            },
            &(size.width(), size.height()),
        );

        state
            .buffer_pool
            .initialize(|| BufferPool::from_device(device));

        state.tile_view_pattern.initialize(|| {
            let tile_view_buffer_desc = wgpu::BufferDescriptor {
                label: Some("tile view buffer"),
                size: size_of::<ShaderTileMetadata>() as wgpu::BufferAddress
                    * DEFAULT_TILE_VIEW_SIZE,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            };

            TileViewPattern::new(BackingBufferDescriptor::new(
                device.create_buffer(&tile_view_buffer_desc),
                tile_view_buffer_desc.size,
            ))
        });

        state.tile_pipeline.initialize(|| {
            let tile_shader = shaders::TileShader {
                format: settings.texture_format,
            };

            let pipeline = TilePipeline::new(
                *settings,
                tile_shader.describe_vertex(),
                tile_shader.describe_fragment(),
                true,
                false,
                false,
                false,
            )
            .describe_render_pipeline()
            .initialize(device);

            state
                .globals_bind_group
                .initialize(|| Globals::from_device(device, &pipeline.get_bind_group_layout(0)));

            pipeline
        });

        state.mask_pipeline.initialize(|| {
            let mask_shader = shaders::TileMaskShader {
                format: settings.texture_format,
                draw_colors: false,
            };

            TilePipeline::new(
                *settings,
                mask_shader.describe_vertex(),
                mask_shader.describe_fragment(),
                false,
                true,
                false,
                false,
            )
            .describe_render_pipeline()
            .initialize(device)
        });
    }
}
