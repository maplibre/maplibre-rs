//! Prepares GPU-owned resources by initializing them if they are uninitialized or out-of-date.

use std::{borrow::Cow, mem};

use crate::{
    context::MapContext,
    render::{
        eventually::Eventually,
        resource::{BackingBufferDescriptor, RenderPipeline, Texture, TilePipeline},
        settings::Msaa,
        shaders,
        shaders::{Shader, ShaderTileMetadata},
        tile_view_pattern::{TileViewPattern, WgpuTileViewPattern, DEFAULT_TILE_VIEW_PATTERN_SIZE},
        MaskPipeline, Renderer,
    },
    tcs::system::System,
};

#[derive(Default)]
pub struct ResourceSystem;

impl System for ResourceSystem {
    fn name(&self) -> Cow<'static, str> {
        "resource_system".into()
    }

    fn run(
        &mut self,
        MapContext {
            renderer:
                Renderer {
                    settings,
                    device,
                    resources: state,
                    ..
                },
            world,
            ..
        }: &mut MapContext,
    ) {
        let Some((
            tile_view_pattern,
            mask_pipeline
        )) = world.resources.query_mut::<(
            &mut Eventually<WgpuTileViewPattern>,
            &mut Eventually<MaskPipeline>,
        )>() else { return; };

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
                    if surface.is_multisampling_supported(settings.msaa) {
                        settings.msaa
                    } else {
                        Msaa { samples: 1 }
                    },
                    wgpu::TextureUsages::RENDER_ATTACHMENT,
                )
            },
            &(size.width(), size.height()),
        );

        state.multisampling_texture.reinitialize(
            || {
                if settings.msaa.is_multisampling()
                    && surface.is_multisampling_supported(settings.msaa)
                {
                    Some(Texture::new(
                        Some("multisampling texture"),
                        device,
                        surface.surface_format(),
                        size.width(),
                        size.height(),
                        settings.msaa,
                        wgpu::TextureUsages::RENDER_ATTACHMENT,
                    ))
                } else {
                    None
                }
            },
            &(size.width(), size.height()),
        );

        tile_view_pattern.initialize(|| {
            let tile_view_buffer_desc = wgpu::BufferDescriptor {
                label: Some("tile view buffer"),
                size: mem::size_of::<ShaderTileMetadata>() as wgpu::BufferAddress
                    * DEFAULT_TILE_VIEW_PATTERN_SIZE,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            };

            TileViewPattern::new(BackingBufferDescriptor::new(
                device.create_buffer(&tile_view_buffer_desc),
                tile_view_buffer_desc.size,
            ))
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
                surface.is_multisampling_supported(settings.msaa),
                false,
            )
            .describe_render_pipeline()
            .initialize(device);
            MaskPipeline(pipeline)
        });
    }
}
