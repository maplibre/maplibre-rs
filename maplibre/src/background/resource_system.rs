use crate::{
    context::MapContext,
    render::{
        eventually::{Eventually, Eventually::Initialized},
        projection::ProjectionGpuResources,
        resource::{RenderPipeline, TilePipeline},
        shaders::{AtmosphereShader, BackgroundShader, GlobeBackgroundShader, Shader},
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
    let Some((
        background_pipeline,
        globe_background_pipeline,
        atmosphere_pipeline,
        Initialized(projection_resources),
    )) = world.resources.query_mut::<(
        &mut Eventually<BackgroundRenderPipeline>,
        &mut Eventually<GlobeBackgroundRenderPipeline>,
        &mut Eventually<AtmosphereRenderPipeline>,
        &mut Eventually<ProjectionGpuResources>,
    )>()
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

    globe_background_pipeline.initialize(|| {
        let shader = GlobeBackgroundShader {
            format: surface.surface_format(),
        };
        let pipeline = TilePipeline::new(
            "globe_background_pipeline".into(),
            *settings,
            shader.describe_vertex(),
            shader.describe_fragment(),
            true,
            false,
            true,
            false,
            surface.is_multisampling_supported(settings.msaa),
            false,
            false,
        )
        .describe_render_pipeline()
        .initialize_with_prefix_layouts(device, &[projection_resources.bind_group_layout()]);
        GlobeBackgroundRenderPipeline(pipeline)
    });

    atmosphere_pipeline.initialize(|| {
        let shader = AtmosphereShader {
            format: surface.surface_format(),
        };
        let pipeline = TilePipeline::new(
            "atmosphere_pipeline".into(),
            *settings,
            shader.describe_vertex(),
            shader.describe_fragment(),
            false,
            false,
            true,
            false,
            surface.is_multisampling_supported(settings.msaa),
            false,
            false,
        )
        .describe_render_pipeline()
        .initialize_with_prefix_layouts(device, &[projection_resources.bind_group_layout()]);
        AtmosphereRenderPipeline(pipeline)
    });

    Ok(())
}

/// Pipeline drawing a flat fullscreen background.
pub struct BackgroundRenderPipeline(pub wgpu::RenderPipeline);

/// Pipeline drawing background paint on a globe mesh.
pub struct GlobeBackgroundRenderPipeline(pub wgpu::RenderPipeline);

/// Pipeline drawing optional atmospheric scattering.
pub struct AtmosphereRenderPipeline(pub wgpu::RenderPipeline);
