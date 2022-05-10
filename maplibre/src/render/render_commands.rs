use crate::render::buffer_pool::IndexEntry;
use crate::render::render_phase::{
    DrawFunctionId, PhaseItem, RenderCommand, RenderCommandResult, TrackedRenderPass,
};
use crate::render::tile_view_pattern::{TileInView, TileShape};
use crate::render::Eventually::Initialized;
use crate::render::INDEX_FORMAT;
use crate::RenderState;

/*pub struct SetMeshViewBindGroup<const I: usize>;
impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetMeshViewBindGroup<I> {
    #[inline]
    fn render<'w>(view: Entity, _item: P, pass: &mut TrackedRenderPass<'w>) -> RenderCommandResult {
        let (view_uniform, view_lights, mesh_view_bind_group) = view_query.get_inner(view).unwrap();
        pass.set_bind_group(
            I,
            &mesh_view_bind_group.value,
            &[view_uniform.offset, view_lights.offset],
        );

        RenderCommandResult::Success
    }
}*/

impl PhaseItem for TileInView {
    type SortKey = ();

    fn sort_key(&self) -> Self::SortKey {
        ()
    }

    fn draw_function(&self) -> DrawFunctionId {
        todo!()
    }
}

impl PhaseItem for (&IndexEntry, &TileShape) {
    type SortKey = u32;

    fn sort_key(&self) -> Self::SortKey {
        self.0.style_layer.index
    }

    fn draw_function(&self) -> DrawFunctionId {
        todo!()
    }
}

pub struct SetMaskPipeline;
impl<P: PhaseItem> RenderCommand<P> for SetMaskPipeline {
    fn render<'w>(
        state: &RenderState,
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
        state: &RenderState,
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
        state: &RenderState,
        TileInView { shape, fallback }: &TileInView,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Initialized(tile_view_pattern) = &state.tile_view_pattern {
            tracing::trace!("Drawing mask {}", &shape.coords);

            let shape_to_render = fallback.as_ref().unwrap_or(&shape);

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
impl RenderCommand<(&IndexEntry, &TileShape)> for DrawTile {
    fn render<'w>(
        state: &RenderState,
        (entry, shape): &(&IndexEntry, &TileShape),
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

type DrawTiles = (SetTilePipeline, DrawTile);

type DrawMasks = (SetMaskPipeline, DrawMask);
