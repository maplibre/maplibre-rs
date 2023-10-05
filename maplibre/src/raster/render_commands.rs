use crate::{
    raster::resource::RasterResources,
    render::{
        eventually::{Eventually, Eventually::Initialized},
        render_phase::{LayerItem, PhaseItem, RenderCommand, RenderCommandResult},
        resource::TrackedRenderPass,
        tile_view_pattern::WgpuTileViewPattern,
    },
    tcs::world::World,
};

pub struct SetRasterTilePipeline;
impl<P: PhaseItem> RenderCommand<P> for SetRasterTilePipeline {
    fn render<'w>(
        world: &'w World,
        _item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(Initialized(raster_resources)) =
            world.resources.get::<Eventually<RasterResources>>()
        else {
            return RenderCommandResult::Failure;
        };

        pass.set_render_pipeline(raster_resources.pipeline());
        RenderCommandResult::Success
    }
}

pub struct SetRasterViewBindGroup<const I: usize>;
impl<const I: usize> RenderCommand<LayerItem> for SetRasterViewBindGroup<I> {
    fn render<'w>(
        world: &'w World,
        item: &LayerItem,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(Initialized(raster_resources)) =
            world.resources.get::<Eventually<RasterResources>>()
        else {
            return RenderCommandResult::Failure;
        };

        let Some(bind_group) = raster_resources.get_bound_texture(&item.tile.coords) else {
            return RenderCommandResult::Failure;
        };

        pass.set_bind_group(0, bind_group, &[]);
        RenderCommandResult::Success
    }
}

pub struct DrawRasterTile;
impl RenderCommand<LayerItem> for DrawRasterTile {
    fn render<'w>(
        world: &'w World,
        item: &LayerItem,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(Initialized(tile_view_pattern)) =
            world.resources.get::<Eventually<WgpuTileViewPattern>>()
        else {
            return RenderCommandResult::Failure;
        };

        let source_shape = &item.source_shape;

        let reference = source_shape.coords().stencil_reference_value_3d() as u32;

        pass.set_stencil_reference(reference);

        let tile_view_pattern_buffer = source_shape
            .buffer_range()
            .expect("tile_view_pattern needs to be uploaded first"); // FIXME tcs
        pass.set_vertex_buffer(
            0,
            tile_view_pattern.buffer().slice(tile_view_pattern_buffer),
        );

        let tile_view_pattern_buffer = source_shape
            .buffer_range()
            .expect("tile_view_pattern needs to be uploaded first"); // FIXME tcs

        // FIXME tcs: I passin random data here right now, but instead we need the correct metadata here
        pass.set_vertex_buffer(
            1,
            tile_view_pattern.buffer().slice(tile_view_pattern_buffer),
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
