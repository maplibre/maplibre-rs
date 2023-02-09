use crate::{
    environment::Environment,
    io::source_client::{HttpSourceClient, SourceClient},
};

/// Holds references to core constructs of maplibre. Based on the compile-time initialization
/// different implementations for handling windows, asynchronous work, or data sources are provided
/// through a [`Kernel`].
///
/// An [`Environment`] defines the types which are used.
///
/// A Kernel lives as long as a [Map](crate::map::Map) usually. It is shared through out various
/// components of the maplibre library.
pub struct Kernel<E: Environment> {
    map_window_config: E::MapWindowConfig,
    apc: E::AsyncProcedureCall,
    scheduler: E::Scheduler,
    source_client: SourceClient<E::HttpClient>,
}

impl<E: Environment> Kernel<E> {
    pub fn map_window_config(&self) -> &E::MapWindowConfig {
        &self.map_window_config
    }

    pub fn apc(&self) -> &E::AsyncProcedureCall {
        &self.apc
    }

    pub fn scheduler(&self) -> &E::Scheduler {
        &self.scheduler
    }

    pub fn source_client(&self) -> &SourceClient<E::HttpClient> {
        &self.source_client
    }
}

/// A convenient builder for [Kernels](Kernel).
pub struct KernelBuilder<E: Environment> {
    map_window_config: Option<E::MapWindowConfig>,
    apc: Option<E::AsyncProcedureCall>,
    scheduler: Option<E::Scheduler>,
    http_client: Option<E::HttpClient>,
}

impl<E: Environment> Default for KernelBuilder<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Environment> KernelBuilder<E> {
    pub fn new() -> Self {
        Self {
            scheduler: None,
            apc: None,
            http_client: None,
            map_window_config: None,
        }
    }

    pub fn with_map_window_config(mut self, map_window_config: E::MapWindowConfig) -> Self {
        self.map_window_config = Some(map_window_config);
        self
    }

    pub fn with_scheduler(mut self, scheduler: E::Scheduler) -> Self {
        self.scheduler = Some(scheduler);
        self
    }

    pub fn with_apc(mut self, apc: E::AsyncProcedureCall) -> Self {
        self.apc = Some(apc);
        self
    }

    pub fn with_http_client(mut self, http_client: E::HttpClient) -> Self {
        self.http_client = Some(http_client);
        self
    }

    pub fn build(self) -> Kernel<E> {
        Kernel {
            scheduler: self.scheduler.unwrap(), // TODO: Remove unwrap
            apc: self.apc.unwrap(),             // TODO: Remove unwrap
            source_client: SourceClient::new(HttpSourceClient::new(self.http_client.unwrap())), // TODO: Remove unwrap
            map_window_config: self.map_window_config.unwrap(), // TODO: Remove unwrap
        }
    }
}
