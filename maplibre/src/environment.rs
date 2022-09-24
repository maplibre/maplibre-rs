use crate::{
    io::{
        apc::AsyncProcedureCall,
        transferables::{
            DefaultTessellatedLayer, DefaultTileTessellated, DefaultUnavailableLayer, Transferables,
        },
    },
    HttpClient, MapWindowConfig, Scheduler,
};

pub trait Environment: 'static {
    type MapWindowConfig: MapWindowConfig;

    type AsyncProcedureCall: AsyncProcedureCall<Self::Transferables, Self::HttpClient>;
    type Scheduler: Scheduler;
    type HttpClient: HttpClient;

    type Transferables: Transferables;
}
