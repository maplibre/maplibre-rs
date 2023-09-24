//! Specifies the instructions which are going to be sent to the GPU. Render commands can be concatenated
//! into a new render command which executes multiple instruction sets.
use crate::{
    debug::{DebugPipeline, TileDebugItem},
    render::{
        eventually::{Eventually, Eventually::Initialized},
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
        let Some(Initialized(pipeline)) = world.resources.get::<Eventually<DebugPipeline>>() else {
            return RenderCommandResult::Failure;
        };

        pass.set_render_pipeline(pipeline);
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

        const TILE_MASK_SHADER_VERTICES: u32 = 24;
        pass.draw(0..TILE_MASK_SHADER_VERTICES, 0..1);

        RenderCommandResult::Success
    }
}

pub type DrawDebugOutlines = (SetDebugPipeline, DrawDebugOutline);
