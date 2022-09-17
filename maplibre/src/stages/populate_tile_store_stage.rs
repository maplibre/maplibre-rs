//! Receives data from async threads and populates the [`crate::io::tile_repository::TileRepository`].

use std::{borrow::BorrowMut, cell::RefCell, ops::Deref, rc::Rc};

use crate::{
    context::MapContext,
    io::{
        apc::{AsyncProcedureCall, Message},
        tile_repository::StoredLayer,
        transferables::{TessellatedLayer, TileTessellated, UnavailableLayer},
    },
    schedule::Stage,
    Environment,
};

pub struct PopulateTileStore<E: Environment> {
    apc: Rc<RefCell<E::AsyncProcedureCall>>,
}

impl<E: Environment> PopulateTileStore<E> {
    pub fn new(apc: Rc<RefCell<E::AsyncProcedureCall>>) -> Self {
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
        if let Ok(mut apc) = self.apc.deref().try_borrow_mut() {
            if let Some(result) = apc.receive() {
                match result {
                    Message::TileTessellated(tranferred) => {
                        let coords = tranferred.coords();
                        tile_repository.success(coords);
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
                        tile_repository.put_tessellated_layer(layer);
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
                        tile_repository.put_tessellated_layer(layer);
                    }
                }
            }
        }
    }
}
