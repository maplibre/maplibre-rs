//! Specifies the instructions which are going to be sent to the GPU. Render commands can be concatenated
//! into a new render command which executes multiple instruction sets.

use crate::render::{
    self,
    eventually::Eventually::Initialized,
    render_phase::{PhaseItem, RenderCommand, RenderCommandResult},
    resource::{Globals, IndexEntry, TrackedRenderPass},
    tile_view_pattern::TileShape,
    RenderState, INDEX_FORMAT,
};

impl PhaseItem for TileShape {
    type SortKey = ();

    fn sort_key(&self) -> Self::SortKey {}
}

impl PhaseItem for (IndexEntry, TileShape) {
    type SortKey = u32;

    fn sort_key(&self) -> Self::SortKey {
        self.0.style_layer.index
    }
}

pub struct SetVectorViewBindGroup<const I: usize>;
impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetVectorViewBindGroup<I> {
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

pub struct SetRasterViewBindGroup<const I: usize>;
impl<const I: usize> RenderCommand<(IndexEntry, TileShape)> for SetRasterViewBindGroup<I> {
    fn render<'w>(
        state: &'w RenderState,
        (entry, _shape): &(IndexEntry, TileShape),
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Initialized(raster_resources) = &state.raster_resources {
            pass.set_bind_group(
                0,
                raster_resources.bind_groups.get(&entry.coords).unwrap(),
                &[],
            );
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
        let Initialized(pipeline) = &state.mask_pipeline  else { return RenderCommandResult::Failure; };
        pass.set_render_pipeline(pipeline);
        RenderCommandResult::Success
    }
}

pub struct SetDebugPipeline;
impl<P: PhaseItem> RenderCommand<P> for SetDebugPipeline {
    fn render<'w>(
        state: &'w RenderState,
        _item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Initialized(pipeline) = &state.debug_pipeline  else { return RenderCommandResult::Failure; };
        pass.set_render_pipeline(pipeline);
        RenderCommandResult::Success
    }
}

pub struct SetVectorTilePipeline;
impl<P: PhaseItem> RenderCommand<P> for SetVectorTilePipeline {
    fn render<'w>(
        state: &'w RenderState,
        _item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Initialized(pipeline) = &state.vector_tile_pipeline  else { return RenderCommandResult::Failure; };
        pass.set_render_pipeline(pipeline);
        RenderCommandResult::Success
    }
}

pub struct SetRasterTilePipeline;
impl<P: PhaseItem> RenderCommand<P> for SetRasterTilePipeline {
    fn render<'w>(
        state: &'w RenderState,
        _item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Initialized(raster_resources) = &state.raster_resources {
            pass.set_render_pipeline(raster_resources.pipeline.as_ref().unwrap());
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Failure
        }
    }
}

pub struct DrawMask;
impl RenderCommand<TileShape> for DrawMask {
    fn render<'w>(
        state: &'w RenderState,
        source_shape: &TileShape,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Initialized(tile_view_pattern) = &state.tile_view_pattern  else { return RenderCommandResult::Failure; };
        tracing::trace!("Drawing mask {}", &source_shape.coords());

        // Draw mask with stencil value of e.g. parent
        let reference = tile_view_pattern.stencil_reference_value_3d(&source_shape.coords()) as u32;

        pass.set_stencil_reference(reference);
        pass.set_vertex_buffer(
            0,
            // Mask is of the requested shape
            tile_view_pattern
                .buffer()
                .slice(source_shape.buffer_range()),
        );
        const TILE_MASK_SHADER_VERTICES: u32 = 6;
        pass.draw(0..TILE_MASK_SHADER_VERTICES, 0..1);

        RenderCommandResult::Success
    }
}

pub struct DrawDebugOutline;
impl RenderCommand<TileShape> for DrawDebugOutline {
    fn render<'w>(
        state: &'w RenderState,
        source_shape: &TileShape,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Initialized(tile_view_pattern) = &state.tile_view_pattern  else { return RenderCommandResult::Failure; };
        pass.set_vertex_buffer(
            0,
            tile_view_pattern
                .buffer()
                .slice(source_shape.buffer_range()),
        );
        const TILE_MASK_SHADER_VERTICES: u32 = 24;
        pass.draw(0..TILE_MASK_SHADER_VERTICES, 0..1);

        RenderCommandResult::Success
    }
}

pub struct DrawVectorTile;
impl RenderCommand<(IndexEntry, TileShape)> for DrawVectorTile {
    fn render<'w>(
        state: &'w RenderState,
        (entry, shape): &(IndexEntry, TileShape),
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let (Initialized(buffer_pool), Initialized(tile_view_pattern)) =
            (&state.buffer_pool, &state.tile_view_pattern) else { return RenderCommandResult::Failure; };

        // Uses stencil value of requested tile and the shape of the requested tile
        let reference = tile_view_pattern.stencil_reference_value_3d(&shape.coords()) as u32;

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

pub struct DrawRasterTile;
impl RenderCommand<(IndexEntry, TileShape)> for DrawRasterTile {
    fn render<'w>(
        state: &'w RenderState,
        (entry, shape): &(IndexEntry, TileShape),
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let (
            Initialized(buffer_pool),
            Initialized(tile_view_pattern),
            Initialized(raster_resources),
        ) = (
            &state.buffer_pool,
            &state.tile_view_pattern,
            &state.raster_resources,
        ) {
            let reference = tile_view_pattern.stencil_reference_value(&shape.coords) as u32;

            tracing::trace!("Drawing raster layer");

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
            pass.draw_indexed(0..render::resource::INDICES.len() as u32, 0, 0..1);
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Failure
        }
    }
}

pub type DrawRasterTiles = (
    SetRasterTilePipeline,
    SetRasterViewBindGroup<0>,
    DrawRasterTile,
);

pub type DrawVectorTiles = (
    SetVectorTilePipeline,
    SetVectorViewBindGroup<0>,
    DrawVectorTile,
);

pub type DrawMasks = (SetMaskPipeline, DrawMask);

pub type DrawDebugOutlines = (SetDebugPipeline, DrawDebugOutline);
