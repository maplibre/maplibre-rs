//! Prepares GPU-owned resources by initializing them if they are uninitialized or out-of-date.
use crate::{
    context::MapContext,
    render::{
        eventually::Eventually,
        resource::{RenderPipeline, TilePipeline},
        shaders,
        shaders::Shader,
        RenderResources, Renderer,
    },
    tcs::system::{SystemError, SystemResult},
    vector::{resource::BufferPool, VectorBufferPool, VectorPipeline},
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
) -> SystemResult {
    let Some((buffer_pool, vector_pipeline)) = world.resources.query_mut::<(
        &mut Eventually<VectorBufferPool>,
        &mut Eventually<VectorPipeline>,
    )>() else {
        return Err(SystemError::Dependencies);
    };

    buffer_pool.initialize(|| BufferPool::from_device(device));

    vector_pipeline.initialize(|| {
        let tile_shader = shaders::FillShader {
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
            false,
        )
        .describe_render_pipeline()
        .initialize(device);

        VectorPipeline(pipeline)
    });

    Ok(())
}
