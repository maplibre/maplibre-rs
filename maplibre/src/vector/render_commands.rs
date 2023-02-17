//! Specifies the instructions which are going to be sent to the GPU. Render commands can be concatenated
//! into a new render command which executes multiple instruction sets.
use crate::{
    ecs::world::World,
    render::{
        eventually::{Eventually, Eventually::Initialized},
        render_phase::{
            LayerItem, PhaseItem, RenderCommand, RenderCommandResult, TileDebugItem, TileMaskItem,
        },
        resource::TrackedRenderPass,
        tile_view_pattern::WgpuTileViewPattern,
        RenderState, INDEX_FORMAT,
    },
    vector::{DebugPipeline, MaskPipeline, VectorBufferPool, VectorPipeline},
};

pub struct SetMaskPipeline;
impl<P: PhaseItem> RenderCommand<P> for SetMaskPipeline {
    fn render<'w>(
        state: &'w RenderState,
        world: &'w World,
        _item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(Initialized(pipeline)) = world.resources.get::<Eventually<MaskPipeline>>() else { return RenderCommandResult::Failure; };
        pass.set_render_pipeline(pipeline);
        RenderCommandResult::Success
    }
}

pub struct SetDebugPipeline;
impl<P: PhaseItem> RenderCommand<P> for SetDebugPipeline {
    fn render<'w>(
        state: &'w RenderState,
        world: &'w World,
        _item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(Initialized(pipeline)) = world
            .resources
            .get::<Eventually<DebugPipeline>>() else { return RenderCommandResult::Failure; };

        pass.set_render_pipeline(pipeline);
        RenderCommandResult::Success
    }
}

pub struct SetVectorTilePipeline;
impl<P: PhaseItem> RenderCommand<P> for SetVectorTilePipeline {
    fn render<'w>(
        state: &'w RenderState,
        world: &'w World,
        _item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(Initialized(pipeline)) = world
            .resources
            .get::<Eventually<VectorPipeline>>() else { return RenderCommandResult::Failure; };

        pass.set_render_pipeline(pipeline);
        RenderCommandResult::Success
    }
}

pub struct DrawMask;
impl RenderCommand<TileMaskItem> for DrawMask {
    fn render<'w>(
        state: &'w RenderState,
        world: &'w World,
        item: &TileMaskItem,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(Initialized(tile_view_pattern)) = world
            .resources
            .get::<Eventually<WgpuTileViewPattern>>() else { return RenderCommandResult::Failure; };

        let tile_mask = &item.source_shape;

        tracing::trace!("Drawing mask {}", &tile_mask.coords());

        // Draw mask with stencil value of e.g. parent
        let reference = tile_mask.coords().stencil_reference_value_3d() as u32;

        pass.set_stencil_reference(reference);

        let tile_view_pattern_buffer = tile_mask
            .buffer_range()
            .expect("tile_view_pattern needs to be uploaded first"); // FIXME: tcs
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

pub struct DrawDebugOutline;
impl RenderCommand<TileDebugItem> for DrawDebugOutline {
    fn render<'w>(
        state: &'w RenderState,
        world: &'w World,
        item: &TileDebugItem,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(Initialized(tile_view_pattern)) = world
            .resources
            .get::<Eventually<WgpuTileViewPattern>>() else { return RenderCommandResult::Failure; };

        let source_shape = &item.source_shape;

        let tile_view_pattern_buffer = source_shape
            .buffer_range()
            .expect("tile_view_pattern needs to be uploaded first"); // FIXME: tcs
        pass.set_vertex_buffer(
            0,
            tile_view_pattern.buffer().slice(tile_view_pattern_buffer),
        );

        const TILE_MASK_SHADER_VERTICES: u32 = 24;
        pass.draw(0..TILE_MASK_SHADER_VERTICES, 0..1);

        RenderCommandResult::Success
    }
}

pub struct DrawVectorTile;
impl RenderCommand<LayerItem> for DrawVectorTile {
    fn render<'w>(
        state: &'w RenderState,
        world: &'w World,
        item: &LayerItem,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some((
            Initialized(buffer_pool),
            Initialized(tile_view_pattern),
        )) = world.resources.query::<(
            &Eventually<VectorBufferPool>,
            &Eventually<WgpuTileViewPattern>
        )>() else { return RenderCommandResult::Failure; };

        let Some(vector_layers) = buffer_pool.index().get_layers(item.tile.coords) else { return RenderCommandResult::Failure; };

        let Some(entry) = vector_layers
            .iter()
            .find(|entry| entry.style_layer.id == item.style_layer) else { return RenderCommandResult::Failure; };

        let source_shape = &item.source_shape;

        // Uses stencil value of requested tile and the shape of the requested tile
        let reference = source_shape.coords().stencil_reference_value_3d() as u32;

        tracing::trace!(
            "Drawing layer {:?} at {}",
            entry.style_layer.source_layer,
            &entry.coords
        );

        let index_range = entry.indices_buffer_range();

        if index_range.is_empty() {
            tracing::error!("Tried to draw a vector tile without any vertices");
            return RenderCommandResult::Failure;
        }

        pass.set_stencil_reference(reference);

        pass.set_index_buffer(buffer_pool.indices().slice(index_range), INDEX_FORMAT);
        pass.set_vertex_buffer(
            0,
            buffer_pool.vertices().slice(entry.vertices_buffer_range()),
        );
        let tile_view_pattern_buffer = source_shape
            .buffer_range()
            .expect("tile_view_pattern needs to be uploaded first"); // FIXME: tcs
        pass.set_vertex_buffer(
            1,
            tile_view_pattern.buffer().slice(tile_view_pattern_buffer),
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
    }
}

pub type DrawVectorTiles = (SetVectorTilePipeline, DrawVectorTile);

pub type DrawMasks = (SetMaskPipeline, DrawMask);

pub type DrawDebugOutlines = (SetDebugPipeline, DrawDebugOutline);
