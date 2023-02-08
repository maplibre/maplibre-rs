use std::{borrow::Cow, rc::Rc};

use crate::{
    context::MapContext,
    ecs::{system::System, Mut},
    environment::Environment,
    io::{
        apc::{AsyncProcedureCall, Message},
        transferables::{LayerRaster, LayerTessellated, LayerUnavailable},
    },
    kernel::Kernel,
    raster::{RasterLayerData, RasterLayersDataComponent},
    vector::{VectorLayerData, VectorLayersDataComponent},
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
                Message::LayerRaster(_) | Message::LayerUnavailable(_) // FIXME: Change to RasterLayerUnavailable
            )
        }) {
            match result {
                Message::LayerRaster(message) => {
                    let layer = message.to_layer();
                    tracing::debug!(
                        "Raster layer {} at {} reached main thread",
                        &layer.source_layer,
                        &layer.coords
                    );
                    log::warn!(
                        "Raster layer {} at {} reached main thread",
                        &layer.source_layer,
                        &layer.coords
                    );
                    tiles
                        .query_component_mut::<Mut<RasterLayersDataComponent>>(layer.coords)
                        .unwrap()
                        .layers
                        .push(layer);
                }
                // FIXME: Change to RasterLayerUnvailable
                Message::LayerUnavailable(message) => {
                    let layer = message.to_layer();

                    tracing::debug!(
                        "Layer {} at {} did not reach main thread",
                        &layer.source_layer,
                        &layer.coords
                    );

                    tiles
                        // FIXME: Change to RasterLayersDataComponent
                        .query_component_mut::<Mut<VectorLayersDataComponent>>(layer.coords)
                        .unwrap()
                        .layers
                        .push(VectorLayerData::Unavailable(layer));
                }
                _ => {}
            }
        }
    }
}
