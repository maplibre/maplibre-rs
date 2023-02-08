use std::{borrow::Cow, rc::Rc};

use crate::{
    context::MapContext,
    ecs::system::System,
    environment::Environment,
    io::{
        apc::{AsyncProcedureCall, Message},
        transferables::{
            LayerIndexed, LayerRaster, LayerTessellated, LayerUnavailable, TileTessellated,
        },
    },
    kernel::Kernel,
    vector::{UnavailableVectorLayerData, VectorLayerData, VectorLayersDataComponent},
};

pub struct PopulateWorldSystem<E: Environment> {
    kernel: Rc<Kernel<E>>,
}

impl<E: Environment> PopulateWorldSystem<E> {
    pub fn new(kernel: &Rc<Kernel<E>>) -> Self {
        Self {
            kernel: kernel.clone(),
        }
    }
}

impl<E: Environment> System for PopulateWorldSystem<E> {
    fn name(&self) -> Cow<'static, str> {
        "populate_world_system".into()
    }

    fn run(
        &mut self,
        MapContext {
            world, renderer, ..
        }: &mut MapContext,
    ) {
        let tiles = &mut world.tiles;
        let geometry_index = &mut world.geometry_index;

        for result in self.kernel.apc().receive(|message| {
            matches!(
                message,
                Message::TileTessellated(_)
                    | Message::LayerTessellated(_)
                    | Message::LayerIndexed(_)
                    | Message::LayerUnavailable(_)
            )
        }) {
            match result {
                // FIXME tcs: deduplicate
                Message::TileTessellated(message) => {
                    let coords = message.coords();
                    tracing::event!(tracing::Level::ERROR, %coords, "tile request done: {}", &coords);

                    tracing::trace!("Vector tile at {} finished loading", coords);
                    log::warn!("Vector tile at {} finished loading", coords);

                    tiles
                        .query_component_mut::<&mut VectorLayersDataComponent>(coords)
                        .unwrap() // FIXME tcs: Unwrap
                        .done = true;
                }
                Message::LayerUnavailable(message) => {
                    let layer = message.to_layer();

                    tracing::debug!(
                        "Source vector layer {} at {} reached main thread",
                        &layer.source_layer,
                        &layer.coords
                    );

                    tiles
                        .query_component_mut::<&mut VectorLayersDataComponent>(layer.coords)
                        .unwrap() // FIXME tcs: Unwrap
                        .layers
                        .push(VectorLayerData::Unavailable(layer));
                }
                Message::LayerTessellated(message) => {
                    // FIXME: Handle points!
                    if message.is_empty() {
                        continue;
                    }

                    let layer = message.to_layer();

                    tracing::debug!(
                        "Source vector layer {} at {} reached main thread",
                        &layer.source_layer,
                        &layer.coords
                    );
                    log::warn!(
                        "Source vector layer {} at {} reached main thread",
                        &layer.source_layer,
                        &layer.coords
                    );

                    tiles
                        .query_component_mut::<&mut VectorLayersDataComponent>(layer.coords)
                        .unwrap() // FIXME tcs: Unwrap
                        .layers
                        .push(VectorLayerData::Available(layer));
                }
                Message::LayerIndexed(message) => {
                    let coords = message.coords();

                    log::warn!(
                        "Source vector layer index at {} reached main thread",
                        coords
                    );

                    geometry_index.index_tile(&coords, message.to_tile_index());
                }
                _ => {}
            }
        }
    }
}
