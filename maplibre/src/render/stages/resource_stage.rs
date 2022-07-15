//! Prepares GPU-owned resources by initializing them if they are uninitialized or out-of-date.

use crate::context::MapContext;

use crate::render::resource::Texture;
use crate::render::resource::{BackingBufferDescriptor, BufferPool};
use crate::render::resource::{Globals, RenderPipeline};
use crate::render::shaders;
use crate::render::shaders::{Shader, ShaderTileMetadata};
use crate::render::tile_pipeline::TilePipeline;
use crate::render::tile_view_pattern::TileViewPattern;
use crate::schedule::Stage;
use crate::Renderer;

use std::mem::size_of;

pub const TILE_VIEW_SIZE: wgpu::BufferAddress = 32;

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
                    wgpu::TextureFormat::Depth24PlusStencil8,
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
                size: size_of::<ShaderTileMetadata>() as wgpu::BufferAddress * TILE_VIEW_SIZE,
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
                settings.msaa,
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
                settings.msaa,
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
