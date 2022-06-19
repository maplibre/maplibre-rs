//! [Stages](Stage) for requesting and preparing data

use crate::schedule::Schedule;

use crate::io::source_client::HttpSourceClient;
use crate::{HttpClient, ScheduleMethod, Scheduler};
use fetch_stage::FetchStage;

mod fetch_stage;
mod message;

/// Register stages required for requesting and preparing new tiles.
pub fn register_stages<HC: HttpClient, SM: ScheduleMethod>(
    schedule: &mut Schedule,
    http_source_client: HttpSourceClient<HC>,
    scheduler: Box<Scheduler<SM>>,
) {
    schedule.add_stage("fetch", FetchStage::new(http_source_client, *scheduler));
}
