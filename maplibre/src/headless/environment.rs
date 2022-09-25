use crate::environment::Environment;
use crate::headless::window::HeadlessMapWindowConfig;
use crate::io::apc::SchedulerAsyncProcedureCall;
use crate::platform::http_client::ReqwestHttpClient;
use crate::platform::scheduler::TokioScheduler;

pub struct HeadlessEnvironment;

impl Environment for HeadlessEnvironment {
    type MapWindowConfig = HeadlessMapWindowConfig;
    type AsyncProcedureCall = SchedulerAsyncProcedureCall<Self::HttpClient, Self::Scheduler>;
    type Scheduler = TokioScheduler;
    type HttpClient = ReqwestHttpClient;
}
