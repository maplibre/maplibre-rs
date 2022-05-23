//! [Stages](Stage) for requesting and preparing data

use crate::io::source_client::SourceClient;
use crate::schedule::Schedule;
use crate::stages::populate_tile_store_stage::PopulateTileStore;
use crate::HTTPClient;
use request_stage::RequestStage;

mod populate_tile_store_stage;
mod request_stage;

pub fn register_stages<HC: HTTPClient>(schedule: &mut Schedule, source_client: SourceClient<HC>) {
    schedule.add_stage("request", RequestStage::new(source_client));
    schedule.add_stage("populate_tile_store", PopulateTileStore::default());
}
