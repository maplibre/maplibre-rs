//! Receives data from async threads and populates the [`crate::io::tile_cache::TileCache`].

use crate::context::MapContext;
use crate::io::{TessellateMessage, TileTessellateMessage};
use crate::schedule::Stage;
use crate::stages::shared_thread_state::SharedThreadState;
use crate::stages::MessageReceiver;
use std::sync::mpsc;

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
    fn run(&mut self, MapContext { tile_cache, .. }: &mut MapContext) {
        if let Ok(result) = self.message_receiver.try_recv() {
            match result {
                TessellateMessage::Layer(layer_result) => {
                    tracing::trace!(
                        "Layer {} at {} reached main thread",
                        layer_result.layer_name(),
                        layer_result.get_coords()
                    );
                    tile_cache.put_tessellated_layer(layer_result);
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
