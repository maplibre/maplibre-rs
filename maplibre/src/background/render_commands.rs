use crate::{
    background::resource_system::{
        AtmosphereRenderPipeline, BackgroundRenderPipeline, GlobeBackgroundRenderPipeline,
    },
    render::{
        eventually::Eventually::{self, Initialized},
        projection::ProjectionGpuResources,
        render_phase::{PhaseItem, RenderCommand, RenderCommandResult},
        resource::TrackedRenderPass,
        tile_mesh::{GlobeTileMeshCache, TileMeshUsage},
    },
    tcs::world::World,
};

pub struct SetBackgroundPipeline;
impl<P: PhaseItem> RenderCommand<P> for SetBackgroundPipeline {
    fn render<'w>(
        world: &'w World,
        _item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(Initialized(BackgroundRenderPipeline(pipeline))) = world
            .resources
            .get::<Eventually<BackgroundRenderPipeline>>()
        else {
            return RenderCommandResult::Failure;
        };

        pass.set_render_pipeline(pipeline);
        RenderCommandResult::Success
    }
}

pub struct DrawBackgroundQuad;
impl<P: PhaseItem> RenderCommand<P> for DrawBackgroundQuad {
    fn render<'w>(
        world: &'w World,
        _item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(buf) = world
            .resources
            .get::<crate::background::queue_system::BackgroundBuffers>()
        {
            pass.set_vertex_buffer(0, buf.metadata_buffer.slice(..));

            pass.draw(0..6, 0..1);
            return RenderCommandResult::Success;
        }
        RenderCommandResult::Failure
    }
}

pub struct SetGlobeBackgroundPipeline;
impl<P: PhaseItem> RenderCommand<P> for SetGlobeBackgroundPipeline {
    fn render<'w>(
        world: &'w World,
        _item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some((
            Initialized(GlobeBackgroundRenderPipeline(pipeline)),
            Initialized(projection_resources),
        )) = world.resources.query::<(
            &Eventually<GlobeBackgroundRenderPipeline>,
            &Eventually<ProjectionGpuResources>,
        )>()
        else {
            return RenderCommandResult::Failure;
        };
        pass.set_render_pipeline(pipeline);
        pass.set_bind_group(0, projection_resources.bind_group(), &[]);
        RenderCommandResult::Success
    }
}

pub struct DrawGlobeBackgroundQuad;
impl<P: PhaseItem> RenderCommand<P> for DrawGlobeBackgroundQuad {
    fn render<'w>(
        world: &'w World,
        _item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some((buffers, mesh_cache)) = world.resources.query::<(
            &crate::background::queue_system::BackgroundBuffers,
            &GlobeTileMeshCache,
        )>() else {
            return RenderCommandResult::Failure;
        };
        let coords = crate::coords::WorldTileCoords::default();
        let Some(mesh) = mesh_cache.get(coords, TileMeshUsage::Raster, false) else {
            return RenderCommandResult::Failure;
        };
        pass.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
        pass.set_vertex_buffer(1, buffers.tile_metadata_buffer.slice(..));
        pass.set_vertex_buffer(2, buffers.metadata_buffer.slice(..));
        pass.set_index_buffer(mesh.index_buffer().slice(..), mesh.index_format());
        pass.draw_indexed(0..mesh.index_count(), 0, 0..1);
        RenderCommandResult::Success
    }
}

pub type DrawGlobeBackground = (SetGlobeBackgroundPipeline, DrawGlobeBackgroundQuad);

pub struct SetAtmospherePipeline;
impl<P: PhaseItem> RenderCommand<P> for SetAtmospherePipeline {
    fn render<'w>(
        world: &'w World,
        _item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some((
            Initialized(AtmosphereRenderPipeline(pipeline)),
            Initialized(projection_resources),
        )) = world.resources.query::<(
            &Eventually<AtmosphereRenderPipeline>,
            &Eventually<ProjectionGpuResources>,
        )>()
        else {
            return RenderCommandResult::Failure;
        };
        pass.set_render_pipeline(pipeline);
        pass.set_bind_group(0, projection_resources.bind_group(), &[]);
        RenderCommandResult::Success
    }
}

pub struct DrawAtmosphereFullscreen;
impl<P: PhaseItem> RenderCommand<P> for DrawAtmosphereFullscreen {
    fn render<'w>(
        world: &'w World,
        _item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(buffers) = world
            .resources
            .get::<crate::background::queue_system::BackgroundBuffers>()
        else {
            return RenderCommandResult::Failure;
        };
        pass.set_vertex_buffer(0, buffers.atmosphere_metadata_buffer.slice(..));
        pass.draw(0..3, 0..1);
        RenderCommandResult::Success
    }
}

pub type DrawAtmosphere = (SetAtmospherePipeline, DrawAtmosphereFullscreen);

pub struct DrawBackground;
impl<P: PhaseItem> RenderCommand<P> for DrawBackground {
    fn render<'w>(
        world: &'w World,
        item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mut result = SetBackgroundPipeline::render(world, item, pass);
        if let RenderCommandResult::Success = result {
            result = DrawBackgroundQuad::render(world, item, pass);
        }
        result
    }
}
