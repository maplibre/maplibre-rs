use crate::io::source_client::SourceClient;
use crate::schedule::Schedule;
use crate::HTTPClient;
use request_stage::RequestStage;

mod request_stage;

pub fn register_stages<HC: HTTPClient>(schedule: &mut Schedule, source_client: SourceClient<HC>) {
    schedule.add_stage("request", RequestStage::new(source_client));
}
