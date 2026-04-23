use crate::{
    context::MapContext,
    render::{
        eventually::Eventually,
        resource::{RenderPipeline, TilePipeline},
        shaders::{BackgroundShader, Shader},
    },
};

pub fn resource_system(
    MapContext {
        world,
        renderer:
            crate::render::Renderer {
                device,
                resources: crate::render::RenderResources { surface, .. },
                settings,
                ..
            },
        ..
    }: &mut MapContext,
) -> crate::tcs::system::SystemResult {
    let Some(background_pipeline) = world
        .resources
        .get_mut::<Eventually<BackgroundRenderPipeline>>()
    else {
        return Err(crate::tcs::system::SystemError::Dependencies);
    };

    background_pipeline.initialize(|| {
        let shader = BackgroundShader {
            format: surface.surface_format(),
        };

        let pipeline = TilePipeline::new(
            "background_pipeline".into(),
            *settings,
            shader.describe_vertex(),
            shader.describe_fragment(),
            true,                                              // depth stencil used
            false,                                             // update stencil
            true,  // debug stencil (Always pass stencil)
            false, // wireframe
            surface.is_multisampling_supported(settings.msaa), // multisampling
            false, // raster
            false, // glyph
        )
        .describe_render_pipeline()
        .initialize(device);

        BackgroundRenderPipeline(pipeline)
    });

    Ok(())
}

pub struct BackgroundRenderPipeline(pub wgpu::RenderPipeline);
