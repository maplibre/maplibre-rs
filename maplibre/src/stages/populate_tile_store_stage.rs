//! Receives data from async threads and populates the [`crate::io::tile_repository::TileRepository`].

use std::rc::Rc;

use crate::{
    context::MapContext,
    environment::Environment,
    io::{
        apc::{AsyncProcedureCall, Message},
        tile_repository::StoredLayer,
        transferables::{LayerIndexed, LayerTessellated, LayerUnavailable, TileTessellated},
    },
    kernel::Kernel,
    schedule::Stage,
    world::World,
};

pub struct PopulateTileStore<E: Environment> {
    kernel: Rc<Kernel<E>>,
}

impl<E: Environment> PopulateTileStore<E> {
    pub fn new(kernel: Rc<Kernel<E>>) -> Self {
        Self { kernel }
    }
}

impl<E: Environment> Stage for PopulateTileStore<E> {
    fn run(
        &mut self,
        MapContext {
            world:
                World {
                    tile_repository,
                    geometry_index,
                    ..
                },
            ..
        }: &mut MapContext,
    ) {
        // TODO: (optimize) Using while instead of if means that we are processing all that is
        // available this might cause frame drops.
        while let Some(result) = self.kernel.apc().receive() {
            match result {
                // TODO: deduplicate
                Message::TileTessellated(message) => {
                    let coords = message.coords();
                    tracing::event!(tracing::Level::ERROR, %coords, "tile request done: {}", &coords);

                    tracing::trace!("Tile at {} finished loading", coords);
                    log::warn!("Tile at {} finished loading", coords);

                    tile_repository.mark_tile_succeeded(&coords).unwrap(); // TODO: unwrap
                }
                Message::LayerUnavailable(message) => {
                    let layer: StoredLayer = message.to_stored_layer();

                    tracing::debug!(
                        "Layer {} at {} reached main thread",
                        layer.layer_name(),
                        layer.get_coords()
                    );

                    tile_repository.put_layer(layer);
                }
                Message::LayerTessellated(message) => {
                    // TODO: Is it fine to ignore layers without any vertices?
                    if message.is_empty() {
                        continue;
                    }

                    let layer: StoredLayer = message.to_stored_layer();

                    tracing::debug!(
                        "Layer {} at {} reached main thread",
                        layer.layer_name(),
                        layer.get_coords()
                    );
                    log::warn!(
                        "Layer {} at {} reached main thread",
                        layer.layer_name(),
                        layer.get_coords()
                    );

                    tile_repository.put_layer(layer);
                }
                Message::LayerIndexed(message) => {
                    let coords = message.coords();

                    log::warn!("Layer index at {} reached main thread", coords);

                    geometry_index.index_tile(&coords, message.to_tile_index());
                }
            }
        }
    }
}
