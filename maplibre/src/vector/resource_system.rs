//! Prepares GPU-owned resources by initializing them if they are uninitialized or out-of-date.
use wgpu::util::{DeviceExt, TextureDataOrder};
use crate::{
    context::MapContext,
    render::{
        eventually::Eventually,
        resource::{RenderPipeline, TilePipeline},
        shaders,
        shaders::Shader,
        RenderResources, Renderer,
    },
    vector::{resource::BufferPool, VectorBufferPool, VectorPipeline},
};
use crate::vector::{SymbolBufferPool, SymbolPipeline};
use crate::vector::resource::GlyphTexture;
use crate::vector::text::GlyphSet;

pub fn resource_system(
    MapContext {
        world,
        renderer:
            Renderer {
                device,
                queue,
                resources: RenderResources { surface, .. },
                settings,
                ..
            },
        ..
    }: &mut MapContext,
) {
    let Some((buffer_pool, vector_pipeline)) = world.resources.query_mut::<(
        &mut Eventually<VectorBufferPool>,
        &mut Eventually<VectorPipeline>,
    )>() else {
        return;
    };

    buffer_pool.initialize(|| BufferPool::from_device(device));

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
            surface.is_multisampling_supported(settings.msaa),
            false,
            false
        )
        .describe_render_pipeline()
        .initialize(device);

        VectorPipeline(pipeline)
    });


    let Some((symbol_buffer_pool, symbol_pipeline, glyph_texture_sampler, glyph_texture_bind_group)) = world.resources.query_mut::<(
        &mut Eventually<SymbolBufferPool>,
        &mut Eventually<SymbolPipeline>,
        &mut Eventually<(wgpu::Texture, wgpu::Sampler)>,
        &mut Eventually<GlyphTexture>,
    )>() else {
        return;
    };

    symbol_buffer_pool.initialize(|| BufferPool::from_device(device));

    symbol_pipeline.initialize(|| {
        let tile_shader = shaders::SymbolTileShader {
            format: surface.surface_format(),
        };

        let pipeline = TilePipeline::new(
            "symbol_pipeline".into(),
            *settings,
            tile_shader.describe_vertex(),
            tile_shader.describe_fragment(),
            true,
            false,
            true, // TODO ignore tile mask
            false,
            surface.is_multisampling_supported(settings.msaa),
            false,
            true
        )
            .describe_render_pipeline()
            .initialize(device);


        let (texture, sampler) = glyph_texture_sampler.initialize(|| {
            let data = std::fs::read("./data/0-255.pbf").unwrap();
            let glyphs = GlyphSet::try_from(
               data.as_slice(),
            ).unwrap();

            let (width, height) = glyphs.get_texture_dimensions();

            let texture = device.create_texture_with_data(
                &queue,
                &wgpu::TextureDescriptor {
                    label: Some("Glyph Texture"),
                    size: wgpu::Extent3d {
                        width: width as _,
                        height: height as _,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::R8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[wgpu::TextureFormat::R8Unorm], // TODO
                },
                TextureDataOrder::LayerMajor, // TODO
                glyphs.get_texture_bytes(),
            );

            let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                // SDF rendering requires linear interpolation
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            });

            (texture, sampler)
        });

        glyph_texture_bind_group.initialize(|| {
            GlyphTexture::from_device(
                device,
                texture,
                sampler,
                &pipeline.get_bind_group_layout(0),
            )
        });

        SymbolPipeline(pipeline)
    });
}
