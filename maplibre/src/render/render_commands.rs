//! Specifies the instructions which are going to be sent to the GPU. Render commands can be concatenated
//! into a new render command which executes multiple instruction sets.
use crate::{
    render::{
        eventually::{Eventually, Eventually::Initialized},
        projection::ProjectionGpuResources,
        render_phase::{PhaseItem, RenderCommand, RenderCommandResult, TileMaskItem},
        resource::TrackedRenderPass,
        tile_mesh::{GlobeTileMeshCache, TileMeshUsage},
        tile_view_pattern::WgpuTileViewPattern,
        MaskPipeline,
    },
    tcs::world::World,
};

pub struct SetMaskPipeline;
impl<P: PhaseItem> RenderCommand<P> for SetMaskPipeline {
    fn render<'w>(
        world: &'w World,
        _item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some((Initialized(pipeline), Initialized(projection_resources))) =
            world.resources.query::<(
                &Eventually<MaskPipeline>,
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

pub struct DrawMask;
impl RenderCommand<TileMaskItem> for DrawMask {
    fn render<'w>(
        world: &'w World,
        item: &TileMaskItem,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some((Initialized(tile_view_pattern), tile_mesh_cache)) = world
            .resources
            .query::<(&Eventually<WgpuTileViewPattern>, &GlobeTileMeshCache)>()
        else {
            return RenderCommandResult::Failure;
        };

        let tile_mask = &item.source_shape;
        let Some(mesh) = tile_mesh_cache.get(
            tile_mask.coords(),
            TileMeshUsage::Stencil,
            item.generate_borders,
        ) else {
            return RenderCommandResult::Failure;
        };

        // Draw mask with stencil value of e.g. parent
        let reference = tile_mask.coords().stencil_reference_value_3d() as u32;

        pass.set_stencil_reference(reference);

        let tile_view_pattern_buffer = tile_mask
            .buffer_range()
            .expect("tile_view_pattern needs to be uploaded first"); // FIXME tcs
        pass.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
        pass.set_vertex_buffer(
            1,
            tile_view_pattern.buffer().slice(tile_view_pattern_buffer),
        );
        pass.set_index_buffer(mesh.index_buffer().slice(..), mesh.index_format());
        pass.draw_indexed(0..mesh.index_count(), 0, 0..1);

        RenderCommandResult::Success
    }
}

pub type DrawMasks = (SetMaskPipeline, DrawMask);
