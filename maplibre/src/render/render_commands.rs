//! Specifies the instructions which are going to be sent to the GPU. Render commands can be concatenated
//! into a new render command which executes multiple instruction sets.
use crate::{
    render::{
        eventually::{Eventually, Eventually::Initialized},
        render_phase::{PhaseItem, RenderCommand, RenderCommandResult, TileMaskItem},
        resource::TrackedRenderPass,
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
        let Some(Initialized(pipeline)) = world.resources.get::<Eventually<MaskPipeline>>() else {
            return RenderCommandResult::Failure;
        };
        pass.set_render_pipeline(pipeline);
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
        let Some(Initialized(tile_view_pattern)) =
            world.resources.get::<Eventually<WgpuTileViewPattern>>()
        else {
            return RenderCommandResult::Failure;
        };

        let tile_mask = &item.source_shape;

        // Draw mask with stencil value of e.g. parent
        let reference = tile_mask.coords().stencil_reference_value_3d() as u32;

        pass.set_stencil_reference(reference);

        let tile_view_pattern_buffer = tile_mask
            .buffer_range()
            .expect("tile_view_pattern needs to be uploaded first"); // FIXME tcs
        pass.set_vertex_buffer(
            0,
            // Mask is of the requested shape
            tile_view_pattern.buffer().slice(tile_view_pattern_buffer),
        );
        const TILE_MASK_SHADER_VERTICES: u32 = 6;
        pass.draw(0..TILE_MASK_SHADER_VERTICES, 0..1);

        RenderCommandResult::Success
    }
}

pub type DrawMasks = (SetMaskPipeline, DrawMask);
