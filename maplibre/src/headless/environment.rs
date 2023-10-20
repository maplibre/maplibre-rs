use crate::{
    environment::Environment,
    headless::window::HeadlessMapWindowConfig,
    io::apc::SchedulerAsyncProcedureCall,
    platform::{
        http_client::ReqwestHttpClient, scheduler::TokioScheduler,
        ReqwestOffscreenKernelEnvironment,
    },
};

pub struct HeadlessEnvironment;

impl Environment for HeadlessEnvironment {
    type MapWindowConfig = HeadlessMapWindowConfig;
    type AsyncProcedureCall =
        SchedulerAsyncProcedureCall<Self::OffscreenKernelEnvironment, Self::Scheduler>;
    type Scheduler = TokioScheduler;
    type HttpClient = ReqwestHttpClient;
    type OffscreenKernelEnvironment = ReqwestOffscreenKernelEnvironment;
}
