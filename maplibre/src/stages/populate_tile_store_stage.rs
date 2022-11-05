//! Receives data from async threads and populates the [`crate::io::tile_repository::TileRepository`].

use std::rc::Rc;

use crate::{
    context::MapContext,
    environment::Environment,
    io::{
        apc::{AsyncProcedureCall, Message},
        tile_repository::StoredLayer,
        transferables::{TessellatedLayer, TileTessellated, UnavailableLayer},
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
            world: World {
                tile_repository, ..
            },
            ..
        }: &mut MapContext,
    ) {
        if let Some(result) = self.kernel.apc().receive() {
            match result {
                Message::TileTessellated(tranferred) => {
                    let coords = tranferred.coords();
                    tile_repository.mark_tile_succeeded(coords);
                    tracing::trace!("Tile at {} finished loading", coords);
                    log::warn!("Tile at {} finished loading", coords);
                }
                // FIXME: deduplicate
                Message::UnavailableLayer(tranferred) => {
                    let layer: StoredLayer = tranferred.to_stored_layer();
                    tracing::debug!(
                        "Layer {} at {} reached main thread",
                        layer.layer_name(),
                        layer.get_coords()
                    );
                    tile_repository.put_layer(layer);
                }
                Message::TessellatedLayer(data) => {
                    let layer: StoredLayer = data.to_stored_layer();
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
            }
        }
    }
}
