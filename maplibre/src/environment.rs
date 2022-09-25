use crate::event_loop::EventLoopConfig;
use crate::kernel::Kernel;
use crate::{
    io::{
        apc::AsyncProcedureCall,
        scheduler::Scheduler,
        source_client::{HttpClient, HttpSourceClient, SourceClient},
        transferables::{
            DefaultTessellatedLayer, DefaultTileTessellated, DefaultUnavailableLayer, Transferables,
        },
    },
    window::MapWindowConfig,
};

pub trait Environment: 'static {
    type MapWindowConfig: MapWindowConfig;

    type AsyncProcedureCall: AsyncProcedureCall<Self::HttpClient>;
    type Scheduler: Scheduler;
    type HttpClient: HttpClient;
}
