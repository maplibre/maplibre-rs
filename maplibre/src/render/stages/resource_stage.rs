use crate::context::MapContext;
use crate::render::buffer_pool::{BackingBufferDescriptor, BufferPool};
use crate::render::resource::pipeline::RenderPipeline;
use crate::render::resource::texture::Texture;
use crate::render::shaders;
use crate::render::shaders::{Shader, ShaderTileMetadata};
use crate::render::tile_pipeline::TilePipeline;
use crate::render::tile_view_pattern::TileViewPattern;
use crate::schedule::Stage;
use crate::{Renderer, ScheduleMethod};
use std::mem::size_of;

pub const TILE_VIEW_SIZE: wgpu::BufferAddress = 32;

#[derive(Default)]
pub struct ResourceStage;

impl Stage for ResourceStage {
    fn run(
        &mut self,
        MapContext {
            renderer:
                Renderer {
                    settings,
                    device,
                    surface,
                    state,
                    ..
                },
            ..
        }: &mut MapContext,
    ) {
        state
            .render_target
            .initialize(|| surface.create_view(device));

        let size = surface.size();

        state.depth_texture.initialize(|| {
            Texture::new(
                Some("depth texture"),
                device,
                wgpu::TextureFormat::Depth24PlusStencil8,
                size.width(),
                size.height(),
                settings.msaa,
            )
        });

        state.multisampling_texture.initialize(|| {
            if settings.msaa.is_active() {
                Some(Texture::new(
                    Some("multisampling texture"),
                    &device,
                    settings.texture_format,
                    size.width(),
                    size.height(),
                    settings.msaa,
                ))
            } else {
                None
            }
        });

        state
            .buffer_pool
            .initialize(|| BufferPool::from_device(device));

        let tile_view_buffer_desc = wgpu::BufferDescriptor {
            label: Some("tile view buffer"),
            size: size_of::<ShaderTileMetadata>() as wgpu::BufferAddress * TILE_VIEW_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };

        state.tile_view_pattern.initialize(|| {
            TileViewPattern::new(BackingBufferDescriptor::new(
                device.create_buffer(&tile_view_buffer_desc),
                tile_view_buffer_desc.size,
            ))
        });

        let tile_shader = shaders::TileShader {
            format: settings.texture_format,
        };
        let mask_shader = shaders::TileMaskShader {
            format: settings.texture_format,
            draw_colors: false,
        };

        state.tile_pipeline.initialize(|| {
            TilePipeline::new(
                settings.msaa,
                tile_shader.describe_vertex(),
                tile_shader.describe_fragment(),
                false,
                false,
                false,
            )
            .describe_render_pipeline()
            .initialize(device)
        });

        state.mask_pipeline.initialize(|| {
            TilePipeline::new(
                settings.msaa,
                mask_shader.describe_vertex(),
                mask_shader.describe_fragment(),
                true,
                false,
                false,
            )
            .describe_render_pipeline()
            .initialize(device)
        });
    }
}
