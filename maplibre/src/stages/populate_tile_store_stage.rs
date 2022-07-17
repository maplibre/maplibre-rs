//! Receives data from async threads and populates the [`crate::io::tile_repository::TileRepository`].

use super::{MessageReceiver, SharedThreadState, TessellateMessage, TileTessellateMessage};
use crate::{context::MapContext, io::tile_repository::StoredLayer, schedule::Stage};

pub struct PopulateTileStore {
    shared_thread_state: SharedThreadState,
    message_receiver: MessageReceiver,
}

impl PopulateTileStore {
    pub fn new(shared_thread_state: SharedThreadState, message_receiver: MessageReceiver) -> Self {
        Self {
            shared_thread_state,
            message_receiver,
        }
    }
}

impl Stage for PopulateTileStore {
    fn run(
        &mut self,
        MapContext {
            tile_repository, ..
        }: &mut MapContext,
    ) {
        if let Ok(result) = self.message_receiver.try_recv() {
            match result {
                TessellateMessage::Layer(layer_result) => {
                    let layer: StoredLayer = layer_result.into();
                    tracing::trace!(
                        "Layer {} at {} reached main thread",
                        layer.layer_name(),
                        layer.get_coords()
                    );
                    tile_repository.put_tessellated_layer(layer);
                }
                TessellateMessage::Tile(TileTessellateMessage { request_id, coords }) => loop {
                    if let Ok(mut tile_request_state) =
                        self.shared_thread_state.tile_request_state.try_lock()
                    {
                        tile_request_state.finish_tile_request(request_id);
                        tracing::trace!("Tile at {} finished loading", coords);
                        break;
                    }
                },
            }
        }
    }
}
