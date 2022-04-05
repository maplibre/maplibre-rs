use reqwest::{Client, StatusCode};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_middleware_cache::managers::CACacheManager;
use reqwest_middleware_cache::{Cache, CacheMode};
use std::future::Future;

use crate::coords::TileCoords;
use crate::error::Error;
use crate::io::scheduler::{IOScheduler, ThreadLocalTessellatorState};
use crate::io::TileRequestID;

pub struct TokioScheduleMethod;

impl TokioScheduleMethod {
    pub fn new() -> Self {
        Self {}
    }

    pub fn schedule<T>(
        &self,
        scheduler: &IOScheduler,
        future_factory: impl (FnOnce(ThreadLocalTessellatorState) -> T) + Send + 'static,
    ) where
        T: std::future::Future + Send + 'static,
        T::Output: Send + 'static,
    {
        tokio::task::spawn(future_factory(scheduler.new_tessellator_state()));
    }
}
