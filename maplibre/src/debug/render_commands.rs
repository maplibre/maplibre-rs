//! Specifies the instructions which are going to be sent to the GPU. Render commands can be concatenated
//! into a new render command which executes multiple instruction sets.
use crate::{
    debug::{DebugPipeline, TileDebugItem},
    render::{
        eventually::{Eventually, Eventually::Initialized},
        projection::ProjectionGpuResources,
        render_phase::{PhaseItem, RenderCommand, RenderCommandResult},
        resource::TrackedRenderPass,
        tile_view_pattern::WgpuTileViewPattern,
    },
    tcs::world::World,
};

pub struct SetDebugPipeline;
impl<P: PhaseItem> RenderCommand<P> for SetDebugPipeline {
    fn render<'w>(
        world: &'w World,
        _item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some((Initialized(pipeline), Initialized(projection_resources))) =
            world.resources.query::<(
                &Eventually<DebugPipeline>,
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

pub struct DrawDebugOutline;
impl RenderCommand<TileDebugItem> for DrawDebugOutline {
    fn render<'w>(
        world: &'w World,
        item: &TileDebugItem,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(Initialized(tile_view_pattern)) =
            world.resources.get::<Eventually<WgpuTileViewPattern>>()
        else {
            return RenderCommandResult::Failure;
        };

        let source_shape = &item.source_shape;

        let tile_view_pattern_buffer = source_shape
            .buffer_range()
            .expect("tile_view_pattern needs to be uploaded first"); // FIXME tcs
        pass.set_vertex_buffer(
            0,
            tile_view_pattern.buffer().slice(tile_view_pattern_buffer),
        );

        // Four edges, each a strip of 32 quads with six vertices, generated in the shader.
        const DEBUG_OUTLINE_VERTICES: u32 = 4 * 32 * 6;
        pass.draw(0..DEBUG_OUTLINE_VERTICES, 0..1);

        RenderCommandResult::Success
    }
}

pub type DrawDebugOutlines = (SetDebugPipeline, DrawDebugOutline);
