//! Specifies the instructions which are going to be sent to the GPU. Render commands can be concatenated
//! into a new render command which executes multiple instruction sets.

use crate::render::{
    eventually::Eventually::Initialized,
    render_phase::{PhaseItem, RenderCommand, RenderCommandResult},
    resource::{Globals, IndexEntry, TrackedRenderPass},
    tile_view_pattern::{TileInView, TileShape},
    RenderState, INDEX_FORMAT,
};

impl PhaseItem for TileInView {
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
        if let Initialized(Globals { bind_group, .. }) = &state.globals_bind_group {
            pass.set_bind_group(0, bind_group, &[]);
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Failure
        }
    }
}

pub struct SetMaskPipeline;
impl<P: PhaseItem> RenderCommand<P> for SetMaskPipeline {
    fn render<'w>(
        state: &'w RenderState,
        _item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Initialized(pipeline) = &state.mask_pipeline {
            pass.set_render_pipeline(pipeline);
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Failure
        }
    }
}

pub struct SetTilePipeline;
impl<P: PhaseItem> RenderCommand<P> for SetTilePipeline {
    fn render<'w>(
        state: &'w RenderState,
        _item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Initialized(pipeline) = &state.tile_pipeline {
            pass.set_render_pipeline(pipeline);
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Failure
        }
    }
}

pub struct DrawMask;
impl RenderCommand<TileInView> for DrawMask {
    fn render<'w>(
        state: &'w RenderState,
        TileInView { shape, fallback }: &TileInView,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Initialized(tile_view_pattern) = &state.tile_view_pattern {
            tracing::trace!("Drawing mask {}", &shape.coords);

            let shape_to_render = fallback.as_ref().unwrap_or(shape);

            let reference =
                tile_view_pattern.stencil_reference_value(&shape_to_render.coords) as u32;

            pass.set_stencil_reference(reference);
            pass.set_vertex_buffer(
                0,
                tile_view_pattern.buffer().slice(shape.buffer_range.clone()),
            );
            pass.draw(0..6, 0..1);
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Failure
        }
    }
}

pub struct DrawTile;
impl RenderCommand<(IndexEntry, TileShape)> for DrawTile {
    fn render<'w>(
        state: &'w RenderState,
        (entry, shape): &(IndexEntry, TileShape),
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let (Initialized(buffer_pool), Initialized(tile_view_pattern)) =
            (&state.buffer_pool, &state.tile_view_pattern)
        {
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
            pass.set_vertex_buffer(
                1,
                tile_view_pattern.buffer().slice(shape.buffer_range.clone()),
            );
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
        } else {
            RenderCommandResult::Failure
        }
    }
}

pub type DrawTiles = (SetTilePipeline, SetViewBindGroup<0>, DrawTile);

pub type DrawMasks = (SetMaskPipeline, DrawMask);
