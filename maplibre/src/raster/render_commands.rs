use crate::{
    ecs::world::World,
    render::{
        eventually::{Eventually, Eventually::Initialized},
        render_phase::{LayerItem, PhaseItem, RenderCommand, RenderCommandResult},
        resource::{RasterResources, TrackedRenderPass},
        tile_view_pattern::TileViewPattern,
        RenderState,
    },
};

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

pub struct SetRasterViewBindGroup<const I: usize>;
impl<const I: usize> RenderCommand<LayerItem> for SetRasterViewBindGroup<I> {
    fn render<'w>(
        state: &'w RenderState,
        world: &'w World,
        item: &LayerItem,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Initialized(raster_resources) = world.get_resource::<Eventually<RasterResources>>() {
            pass.set_bind_group(
                0,
                raster_resources
                    .get_bound_texture(&item.tile.coords)
                    .unwrap(), // TODO Remove unwrap
                &[],
            );
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Failure
        }
    }
}

pub struct DrawRasterTile;
impl RenderCommand<LayerItem> for DrawRasterTile {
    fn render<'w>(
        state: &'w RenderState,
        world: &'w World,
        item: &LayerItem,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let source_shape = &item.source_shape;
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
