use crate::{
    context::MapContext,
    render::render_phase::{LayerItem, RenderPhase, TileMaskItem, TranslucentItem},
    tcs::system::{SystemError, SystemResult},
};

pub fn cleanup_system(MapContext { world, .. }: &mut MapContext) -> SystemResult {
    let Some((layer_item_phase, tile_mask_phase, translucent_phase)) =
        world.resources.query_mut::<(
            &mut RenderPhase<LayerItem>,
            &mut RenderPhase<TileMaskItem>,
            &mut RenderPhase<TranslucentItem>,
        )>()
    else {
        return Err(SystemError::Dependencies);
    };

    layer_item_phase.clear();
    tile_mask_phase.clear();
    translucent_phase.clear();

    Ok(())
}
