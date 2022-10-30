use crate::{
    event_loop::EventLoopConfig,
    io::{
        apc::AsyncProcedureCall,
        scheduler::Scheduler,
        source_client::{HttpClient, HttpSourceClient, SourceClient},
        transferables::{
            DefaultTessellatedLayer, DefaultTileTessellated, DefaultUnavailableLayer, Transferables,
        },
    },
    kernel::Kernel,
    window::MapWindowConfig,
};

pub trait Environment: 'static {
    type MapWindowConfig: MapWindowConfig;

    type AsyncProcedureCall: AsyncProcedureCall<Self::HttpClient>;

    type Scheduler: Scheduler;

    type HttpClient: HttpClient;
}
