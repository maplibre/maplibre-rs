use crate::{
    environment::Environment,
    headless::window::HeadlessMapWindowConfig,
    io::apc::SchedulerAsyncProcedureCall,
    platform::{
        http_client::ReqwestHttpClient, scheduler::TokioScheduleMethod,
        ReqwestOffscreenKernelEnvironment,
    },
};

pub struct HeadlessEnvironment;

impl Environment for HeadlessEnvironment {
    type MapWindowConfig = HeadlessMapWindowConfig;
    type AsyncProcedureCall =
        SchedulerAsyncProcedureCall<Self::OffscreenKernelEnvironment, Self::Scheduler>;
    type Scheduler = TokioScheduleMethod;
    type HttpClient = ReqwestHttpClient;
    type OffscreenKernelEnvironment = ReqwestOffscreenKernelEnvironment;
}
