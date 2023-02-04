//! Specifies the instructions which are going to be sent to the GPU. Render commands can be concatenated
//! into a new render command which executes multiple instruction sets.

use std::ops::Deref;

use crate::{
    ecs::world::World,
    render::{
        eventually::{Eventually, Eventually::Initialized},
        render_phase::{PhaseItem, RenderCommand, RenderCommandResult},
        resource::{Globals, IndexEntry, RasterResources, TrackedRenderPass},
        tile_view_pattern::{TileShape, TileViewPattern},
        RenderState, INDEX_FORMAT,
    },
    vector::{DebugPipeline, MaskPipeline, MaskRenderPhase, VectorBufferPool, VectorPipeline},
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
        world: &'w World,
        _item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Initialized(Globals { bind_group, .. }) = world.get_resource::<Eventually<Globals>>()  else { return RenderCommandResult::Failure; };
        pass.set_bind_group(0, bind_group, &[]);
        RenderCommandResult::Success
    }
}

pub struct SetRasterViewBindGroup<const I: usize>;
impl<const I: usize> RenderCommand<TileShape> for SetRasterViewBindGroup<I> {
    fn render<'w>(
        state: &'w RenderState,
        world: &'w World,
        shape: &TileShape,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Initialized(raster_resources) = world.get_resource::<Eventually<RasterResources>>() {
            pass.set_bind_group(
                0,
                raster_resources.get_bound_texture(&shape.coords()).unwrap(), // TODO Remove unwrap
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
        world: &'w World,
        _item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Initialized(pipeline) = world.get_resource::<Eventually<MaskPipeline>>()  else { return RenderCommandResult::Failure; };
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
        let Initialized(pipeline) = world.get_resource::<Eventually<DebugPipeline>>()  else { return RenderCommandResult::Failure; };
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
        let Initialized(pipeline) = world.get_resource::<Eventually<VectorPipeline>>()  else { return RenderCommandResult::Failure; };
        pass.set_render_pipeline(pipeline);
        RenderCommandResult::Success
    }
}

pub struct SetRasterTilePipeline;
impl<P: PhaseItem> RenderCommand<P> for SetRasterTilePipeline {
    fn render<'w>(
        state: &'w RenderState,
        world: &'w World,
        _item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Initialized(raster_resources) = world.get_resource::<Eventually<RasterResources>>() {
            pass.set_render_pipeline(raster_resources.pipeline());
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
        world: &'w World,
        source_shape: &TileShape,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Initialized(tile_view_pattern) = world.get_resource::<Eventually<TileViewPattern<wgpu::Queue, wgpu::Buffer>>>()  else { return RenderCommandResult::Failure; };
        tracing::trace!("Drawing mask {}", &source_shape.coords());

        // Draw mask with stencil value of e.g. parent
        let reference = source_shape.coords().stencil_reference_value_3d() as u32;

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
        world: &'w World,
        source_shape: &TileShape,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Initialized(tile_view_pattern) = world.get_resource::<Eventually<TileViewPattern<wgpu::Queue, wgpu::Buffer>>>()  else { return RenderCommandResult::Failure; };
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
        world: &'w World,
        (entry, shape): &(IndexEntry, TileShape),
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let (Initialized(buffer_pool), Initialized(tile_view_pattern)) =
            (
                world.get_resource::<Eventually<VectorBufferPool>>(),
                world.get_resource::<Eventually<TileViewPattern<wgpu::Queue, wgpu::Buffer>>>(),
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

pub struct DrawRasterTile;
impl RenderCommand<TileShape> for DrawRasterTile {
    fn render<'w>(
        state: &'w RenderState,
        world: &'w World,
        source_shape: &TileShape,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Initialized(tile_view_pattern) = world.get_resource::<Eventually<TileViewPattern<wgpu::Queue, wgpu::Buffer>>>() else { return RenderCommandResult::Failure; };

        let reference = source_shape.coords().stencil_reference_value_3d() as u32;

        tracing::trace!("Drawing raster layer");

        pass.set_stencil_reference(reference);

        pass.set_vertex_buffer(
            0,
            tile_view_pattern
                .buffer()
                .slice(source_shape.buffer_range()),
        );

        const TILE_MASK_SHADER_VERTICES: u32 = 6;
        pass.draw(0..TILE_MASK_SHADER_VERTICES, 0..1);

        RenderCommandResult::Success
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
