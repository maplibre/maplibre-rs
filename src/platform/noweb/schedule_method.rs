
use reqwest::{Client, StatusCode};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_middleware_cache::managers::CACacheManager;
use reqwest_middleware_cache::{Cache, CacheMode};
use std::future::Future;

use crate::coords::TileCoords;
use crate::error::Error;
use crate::io::scheduler::IOScheduler;
use crate::io::TileRequestID;

pub struct TokioScheduleMethod;

impl TokioScheduleMethod {
    pub fn new() -> Self {
        Self {}
    }

    pub fn schedule<T>(&self, future: T)
    where
        T: Future<Output = ()> + Send + 'static,
    {
        tokio::task::spawn(future);
    }
}
