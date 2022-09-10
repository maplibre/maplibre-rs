//! Receives data from async threads and populates the [`crate::io::tile_repository::TileRepository`].

use crate::io::apc::{AsyncProcedureCall, Transferable};
use crate::io::transferables::{TessellatedLayer, TileTessellated, UnavailableLayer};
use crate::{context::MapContext, io::tile_repository::StoredLayer, schedule::Stage, Environment};
use std::rc::Rc;

pub struct PopulateTileStore<E: Environment> {
    apc: Rc<E::AsyncProcedureCall>,
}

impl<E: Environment> PopulateTileStore<E> {
    pub fn new(apc: Rc<E::AsyncProcedureCall>) -> Self {
        Self { apc }
    }
}

impl<E: Environment> Stage for PopulateTileStore<E> {
    fn run(
        &mut self,
        MapContext {
            tile_repository, ..
        }: &mut MapContext,
    ) {
        if let Some(result) = self.apc.receive() {
            match result {
                Transferable::TileTessellated(tranferred) => {
                    let coords = tranferred.coords();
                    tile_repository.success(coords);
                    tracing::trace!("Tile at {} finished loading", coords);
                }
                Transferable::UnavailableLayer(tranferred) => {
                    let layer: StoredLayer = tranferred.to_stored_layer();
                    tracing::debug!(
                        "Layer {} at {} reached main thread",
                        layer.layer_name(),
                        layer.get_coords()
                    );
                    tile_repository.put_tessellated_layer(layer);
                }
                Transferable::TessellatedLayer(data) => {
                    let layer: StoredLayer = data.to_stored_layer();
                    tracing::debug!(
                        "Layer {} at {} reached main thread",
                        layer.layer_name(),
                        layer.get_coords()
                    );
                    tile_repository.put_tessellated_layer(layer);
                }
            }
        }
    }
}
