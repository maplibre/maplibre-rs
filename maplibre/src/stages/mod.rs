//! [Stages](Stage) for requesting and preparing data

use crate::io::geometry_index::GeometryIndex;
use crate::io::source_client::{HttpSourceClient, SourceClient};
use crate::io::tile_request_state::TileRequestState;
use crate::io::TessellateMessage;
use crate::schedule::Schedule;
use crate::stages::populate_tile_store_stage::PopulateTileStore;
use crate::stages::shared_thread_state::SharedThreadState;
use crate::{HttpClient, ScheduleMethod, Scheduler};
use request_stage::RequestStage;
use std::sync::{mpsc, Arc, Mutex};

mod populate_tile_store_stage;
mod request_stage;
mod shared_thread_state;

pub type MessageSender = mpsc::Sender<TessellateMessage>;
pub type MessageReceiver = mpsc::Receiver<TessellateMessage>;

pub fn register_stages<HC: HttpClient, SM: ScheduleMethod>(
    schedule: &mut Schedule,
    http_source_client: HttpSourceClient<HC>,
    scheduler: Box<Scheduler<SM>>,
) {
    let (message_sender, message_receiver): (MessageSender, MessageReceiver) = mpsc::channel();
    let shared_thread_state = SharedThreadState {
        tile_request_state: Arc::new(Mutex::new(TileRequestState::new())),
        message_sender,
        geometry_index: Arc::new(Mutex::new(GeometryIndex::new())),
    };

    let scheduler = Box::new(scheduler.take());

    schedule.add_stage(
        "request",
        RequestStage::new(shared_thread_state.clone(), http_source_client, scheduler),
    );
    schedule.add_stage(
        "populate_tile_store",
        PopulateTileStore::new(shared_thread_state, message_receiver),
    );
}
