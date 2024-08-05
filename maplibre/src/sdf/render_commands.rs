
use crate::{
    render::{
        eventually::{Eventually, Eventually::Initialized},
        render_phase::{LayerItem, PhaseItem, RenderCommand, RenderCommandResult},
        resource::TrackedRenderPass,
        tile_view_pattern::WgpuTileViewPattern,
        INDEX_FORMAT,
    },
    tcs::world::World,
};
use crate::sdf::resource::GlyphTexture;
use crate::sdf::{SymbolBufferPool, SymbolPipeline};

pub struct SetSymbolPipeline;
impl<P: PhaseItem> RenderCommand<P> for SetSymbolPipeline {
    fn render<'w>(
        world: &'w World,
        _item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some((Initialized(GlyphTexture { ref bind_group, .. }), Initialized(symbol_pipeline))) =
            world.resources.query::<(
                &Eventually<GlyphTexture>,
                &Eventually<SymbolPipeline>,
            )>()
        else {
            return RenderCommandResult::Failure;
        };

        pass.set_bind_group(0, bind_group, &[]);
        pass.set_render_pipeline(symbol_pipeline);
        RenderCommandResult::Success
    }
}

pub struct DrawSymbol;
impl RenderCommand<LayerItem> for DrawSymbol {
    fn render<'w>(
        world: &'w World,
        item: &LayerItem,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some((Initialized(symbol_buffer_pool), Initialized(tile_view_pattern))) =
            world.resources.query::<(
                &Eventually<SymbolBufferPool>,
                &Eventually<WgpuTileViewPattern>,
            )>()
        else {
            return RenderCommandResult::Failure;
        };

        let Some(vector_layers) = symbol_buffer_pool.index().get_layers(item.tile.coords) else {
            return RenderCommandResult::Failure;
        };

        let Some(entry) = vector_layers
            .iter()
            .find(|entry| entry.style_layer.id == item.style_layer)
        else {
            return RenderCommandResult::Failure;
        };

        let source_shape = &item.source_shape;

        let tile_view_pattern_buffer = source_shape
            .buffer_range()
            .expect("tile_view_pattern needs to be uploaded first"); // FIXME tcs

        // Uses stencil value of requested tile and the shape of the requested tile
        let reference = source_shape.coords().stencil_reference_value_3d() as u32;

        tracing::trace!(
            "Drawing layer {:?} at {}",
            entry.style_layer.source_layer,
            entry.coords
        );

        let index_range = entry.indices_buffer_range();

        if index_range.is_empty() {
            tracing::error!("Tried to draw a vector tile without any vertices");
            return RenderCommandResult::Failure;
        }

        pass.set_stencil_reference(reference);

        pass.set_index_buffer(
            symbol_buffer_pool
                .indices()
                .slice(index_range),
            INDEX_FORMAT,
        );
        pass.set_vertex_buffer(
            0,
            symbol_buffer_pool
                .vertices()
                .slice(entry.vertices_buffer_range()),
        );
        pass.set_vertex_buffer(1, tile_view_pattern.buffer().slice(tile_view_pattern_buffer));
        pass.set_vertex_buffer(
            2,
            symbol_buffer_pool
                .metadata()
                .slice(entry.layer_metadata_buffer_range()),
        );

        pass.draw_indexed(entry.indices_range(), 0, 0..1);
        RenderCommandResult::Success
    }
}


pub type DrawSymbols = (SetSymbolPipeline, DrawSymbol);