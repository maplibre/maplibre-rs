//! Specifies the instructions which are going to be sent to the GPU. Render commands can be concatenated
//! into a new render command which executes multiple instruction sets.

use log::info;

use crate::{
    ecs::{world::World, Ref},
    render::{
        eventually::{Eventually, Eventually::Initialized},
        render_phase::{LayerItem, PhaseItem, RenderCommand, RenderCommandResult, TileMaskItem},
        resource::{IndexEntry, RasterResources, TrackedRenderPass},
        tile_view_pattern::{TileShape, TileViewPattern},
        RenderState, INDEX_FORMAT,
    },
    vector::{
        DebugPipeline, MaskPipeline, VectorBufferPool, VectorLayersIndicesComponent, VectorPipeline,
    },
};

pub struct SetMaskPipeline;
impl<P: PhaseItem> RenderCommand<P> for SetMaskPipeline {
    fn render<'w>(
        state: &'w RenderState,
        world: &'w World,
        _item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Initialized(pipeline) = world.resources.get::<Eventually<MaskPipeline>>().unwrap() else { return RenderCommandResult::Failure; };
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
        let Initialized(pipeline) = world.resources.get::<Eventually<DebugPipeline>>().unwrap() else { return RenderCommandResult::Failure; };
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
        let Initialized(pipeline) = world.resources.get::<Eventually<VectorPipeline>>().unwrap() else { return RenderCommandResult::Failure; };
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
        let tile_mask = &item.source_shape;

        let Initialized(tile_view_pattern) = world.resources.get::<Eventually<TileViewPattern<wgpu::Queue, wgpu::Buffer>>>().unwrap() else { return RenderCommandResult::Failure; };
        tracing::trace!("Drawing mask {}", &tile_mask.coords());

        // Draw mask with stencil value of e.g. parent
        let reference = tile_mask.coords().stencil_reference_value_3d() as u32;

        pass.set_stencil_reference(reference);
        pass.set_vertex_buffer(
            0,
            // Mask is of the requested shape
            tile_view_pattern.buffer().slice(tile_mask.buffer_range()),
        );
        const TILE_MASK_SHADER_VERTICES: u32 = 6;
        pass.draw(0..TILE_MASK_SHADER_VERTICES, 0..1);

        RenderCommandResult::Success
    }
}

pub struct DrawDebugOutline;
impl RenderCommand<LayerItem> for DrawDebugOutline {
    fn render<'w>(
        state: &'w RenderState,
        world: &'w World,
        item: &LayerItem,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let source_shape = &item.source_shape;
        let Initialized(tile_view_pattern) = world.resources.get::<Eventually<TileViewPattern<wgpu::Queue, wgpu::Buffer>>>().unwrap() else { return RenderCommandResult::Failure; };
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
impl RenderCommand<LayerItem> for DrawVectorTile {
    fn render<'w>(
        state: &'w RenderState,
        world: &'w World,
        item: &LayerItem,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let shape = &item.source_shape; // FIXME: Is this correct?
        let vector_layers = world
            .tiles
            .query_component::<Ref<VectorLayersIndicesComponent>>(item.tile.coords)
            .unwrap();

        let entry = &vector_layers
            .layers
            .iter()
            .find(|entry| entry.style_layer.id == item.style_layer)
            .unwrap();

        let (Initialized(buffer_pool), Initialized(tile_view_pattern)) =
            (
                world.resources.get::<Eventually<VectorBufferPool>>().unwrap(),
                world.resources.get::<Eventually<TileViewPattern<wgpu::Queue, wgpu::Buffer>>>().unwrap(),
            ) else { return RenderCommandResult::Failure; };

        // Uses stencil value of requested tile and the shape of the requested tile
        let reference = shape.coords().stencil_reference_value_3d() as u32;

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

pub type DrawVectorTiles = (SetVectorTilePipeline, DrawVectorTile);

pub type DrawMasks = (SetMaskPipeline, DrawMask);

pub type DrawDebugOutlines = (SetDebugPipeline, DrawDebugOutline);
