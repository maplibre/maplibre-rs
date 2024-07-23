use serde::{Deserialize, Serialize};

use crate::{
    io::{
        apc::AsyncProcedureCall,
        scheduler::Scheduler,
        source_client::{HttpClient, SourceClient},
    },
    window::MapWindowConfig,
};

/// The environment defines which types must be injected into maplibre at compile time.
/// Essentially, this trait implements the
/// [dependency injection](https://en.wikipedia.org/wiki/Dependency_injection) design pattern.
/// By instantiating this trait at compile time with concrete types, it is possible to create
/// different compile-time instances of maplibre.
///
/// For example it is possible to change the way tasks are scheduled. It is also possible to change
/// the HTTP implementation for fetching tiles over the network.
pub trait Environment: 'static {
    type MapWindowConfig: MapWindowConfig;

    type AsyncProcedureCall: AsyncProcedureCall<Self::OffscreenKernelEnvironment>;

    type Scheduler: Scheduler;

    type HttpClient: HttpClient;

    type OffscreenKernelEnvironment: OffscreenKernel;
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OffscreenKernelConfig {
    pub cache_directory: Option<String>,
}

pub trait OffscreenKernel: Send + Sync + 'static {
    type HttpClient: HttpClient;
    fn create(config: OffscreenKernelConfig) -> Self;

    fn source_client(&self) -> SourceClient<Self::HttpClient>;
}
