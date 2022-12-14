//! Specifies the instructions which are going to be sent to the GPU. Render commands can be concatenated
//! into a new render command which executes multiple instruction sets.

use crate::render::{
    eventually::Eventually::Initialized,
    render_phase::{PhaseItem, RenderCommand, RenderCommandResult},
    resource::{Globals, IndexEntry, TrackedRenderPass},
    tile_view_pattern::{TileShape, ViewTile},
    RenderState, INDEX_FORMAT,
};

impl PhaseItem for ViewTile {
    type SortKey = ();

    fn sort_key(&self) -> Self::SortKey {}
}

impl PhaseItem for (IndexEntry, TileShape) {
    type SortKey = u32;

    fn sort_key(&self) -> Self::SortKey {
        self.0.style_layer.index
    }
}

pub struct SetViewBindGroup<const I: usize>;
impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetViewBindGroup<I> {
    fn render<'w>(
        state: &'w RenderState,
        _item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Initialized(Globals { bind_group, .. }) = &state.globals_bind_group  else { return RenderCommandResult::Failure; };
        pass.set_bind_group(0, bind_group, &[]);
        RenderCommandResult::Success
    }
}

pub struct SetMaskPipeline;
impl<P: PhaseItem> RenderCommand<P> for SetMaskPipeline {
    fn render<'w>(
        state: &'w RenderState,
        _item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Initialized(pipeline) = &state.mask_pipeline  else { return RenderCommandResult::Failure; };
        pass.set_render_pipeline(pipeline);
        RenderCommandResult::Success
    }
}

pub struct SetDebugMasksPipeline;
impl<P: PhaseItem> RenderCommand<P> for SetDebugMasksPipeline {
    fn render<'w>(
        state: &'w RenderState,
        _item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Initialized(pipeline) = &state.debug_mask_pipeline  else { return RenderCommandResult::Failure; };
        pass.set_render_pipeline(pipeline);
        RenderCommandResult::Success
    }
}

pub struct SetTilePipeline;
impl<P: PhaseItem> RenderCommand<P> for SetTilePipeline {
    fn render<'w>(
        state: &'w RenderState,
        _item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Initialized(pipeline) = &state.tile_pipeline  else { return RenderCommandResult::Failure; };
        pass.set_render_pipeline(pipeline);
        RenderCommandResult::Success
    }
}

pub struct DrawMask;
impl RenderCommand<ViewTile> for DrawMask {
    fn render<'w>(
        state: &'w RenderState,
        view_tile: &ViewTile,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Initialized(tile_view_pattern) = &state.tile_view_pattern  else { return RenderCommandResult::Failure; };
        tracing::trace!("Drawing mask {}", &view_tile.coords());

        view_tile.render(|target_shape, source_shape| {
            // Draw mask with stencil value of e.g. parent
            let reference = tile_view_pattern.stencil_reference_value(&source_shape.coords) as u32;

            pass.set_stencil_reference(reference);
            pass.set_vertex_buffer(
                0,
                // Mask is of the requested shape
                // FIXME (THIS_PR): Use here target or source?
                tile_view_pattern
                    .buffer()
                    .slice(source_shape.buffer_range()),
            );
            const TILE_MASK_SHADER_VERTICES: u32 = 6;
            pass.draw(0..TILE_MASK_SHADER_VERTICES, 0..1);
        });
        RenderCommandResult::Success
    }
}

pub struct DrawDebugMask;
impl RenderCommand<ViewTile> for DrawDebugMask {
    fn render<'w>(
        state: &'w RenderState,
        view_tile: &ViewTile,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Initialized(tile_view_pattern) = &state.tile_view_pattern  else { return RenderCommandResult::Failure; };
        tracing::trace!("Drawing mask {}", &view_tile.coords());

        view_tile.render(|target_shape, source_shape| {
            pass.set_vertex_buffer(
                0,
                tile_view_pattern
                    .buffer()
                    .slice(source_shape.buffer_range()),
            );
            const TILE_MASK_SHADER_VERTICES: u32 = 24;
            pass.draw(0..TILE_MASK_SHADER_VERTICES, 0..1);
        });

        RenderCommandResult::Success
    }
}

pub struct DrawTile;
impl RenderCommand<(IndexEntry, TileShape)> for DrawTile {
    fn render<'w>(
        state: &'w RenderState,
        (entry, shape): &(IndexEntry, TileShape),
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let (Initialized(buffer_pool), Initialized(tile_view_pattern)) =
            (&state.buffer_pool, &state.tile_view_pattern) else { return RenderCommandResult::Failure; };

        // Uses stencil value of requested tile and the shape of the requested tile
        let reference = tile_view_pattern.stencil_reference_value(&shape.coords) as u32;

        tracing::trace!(
            "Drawing layer {:?} at {}",
            entry.style_layer.source_layer,
            &entry.coords
        );

        pass.set_stencil_reference(reference);
        pass.set_index_buffer(
            buffer_pool.indices().slice(entry.indices_buffer_range()),
            INDEX_FORMAT,
        );
        pass.set_vertex_buffer(
            0,
            buffer_pool.vertices().slice(entry.vertices_buffer_range()),
        );
        pass.set_vertex_buffer(1, tile_view_pattern.buffer().slice(shape.buffer_range()));
        pass.set_vertex_buffer(
            2,
            buffer_pool
                .metadata()
                .slice(entry.layer_metadata_buffer_range()),
        );
        pass.set_vertex_buffer(
            3,
            buffer_pool
                .feature_metadata()
                .slice(entry.feature_metadata_buffer_range()),
        );
        pass.draw_indexed(entry.indices_range(), 0, 0..1);
        RenderCommandResult::Success
    }
}

pub type DrawTiles = (SetTilePipeline, SetViewBindGroup<0>, DrawTile);

pub type DrawMasks = (SetMaskPipeline, DrawMask);
pub type DrawDebugMasks = (SetDebugMasksPipeline, DrawDebugMask);
