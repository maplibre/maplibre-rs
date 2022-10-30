use crate::{
    environment::Environment,
    io::source_client::{HttpSourceClient, SourceClient},
};

pub struct Kernel<E: Environment> {
    pub map_window_config: E::MapWindowConfig,
    pub apc: E::AsyncProcedureCall,
    pub scheduler: E::Scheduler,
    pub source_client: SourceClient<E::HttpClient>,
}

pub struct KernelBuilder<E: Environment> {
    map_window_config: Option<E::MapWindowConfig>,
    apc: Option<E::AsyncProcedureCall>,
    scheduler: Option<E::Scheduler>,
    http_client: Option<E::HttpClient>,
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
