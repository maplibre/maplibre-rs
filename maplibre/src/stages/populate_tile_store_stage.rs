//! Receives data from async threads and populates the [`crate::io::tile_cache::TileCache`].

use crate::context::MapContext;
use crate::io::{TessellateMessage, TileTessellateMessage};
use crate::schedule::Stage;

#[derive(Default)]
pub struct PopulateTileStore {}

impl Stage for PopulateTileStore {
    fn run(
        &mut self,
        MapContext {
            tile_cache,
            shared_thread_state,
            message_receiver,
            ..
        }: &mut MapContext,
    ) {
        if let Ok(result) = message_receiver.try_recv() {
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
                        shared_thread_state.tile_request_state.try_lock()
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
