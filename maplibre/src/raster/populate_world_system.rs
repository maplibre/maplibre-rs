use std::{borrow::Cow, rc::Rc};

use crate::{
    context::MapContext,
    ecs::system::System,
    environment::Environment,
    io::{
        apc::{AsyncProcedureCall, Message},
        tile_repository::StoredLayer,
        transferables::{LayerRaster, LayerTessellated, LayerUnavailable},
    },
    kernel::Kernel,
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
        let tile_repository = &mut world.tile_repository;
        let geometry_index = &mut world.geometry_index;

        for result in self.kernel.apc().receive(|message| {
            matches!(
                message,
                Message::LayerRaster(_) | Message::LayerUnavailable(_) // FIXME: Change to RasterLayerUnavailable
            )
        }) {
            match result {
                Message::LayerRaster(message) => {
                    let layer: StoredLayer = message.to_stored_layer();
                    tracing::debug!(
                        "Raster layer {} at {} reached main thread",
                        layer.layer_name(),
                        layer.get_coords()
                    );
                    log::warn!(
                        "Raster layer {} at {} reached main thread",
                        layer.layer_name(),
                        layer.get_coords()
                    );
                    tile_repository.put_layer(layer);
                }
                Message::LayerUnavailable(message) => {
                    let layer: StoredLayer = message.to_stored_layer();

                    tracing::debug!(
                        "Layer {} at {} did not reach main thread",
                        layer.layer_name(),
                        layer.get_coords()
                    );

                    tile_repository.put_layer(layer);
                }
                _ => {}
            }
        }
    }
}
