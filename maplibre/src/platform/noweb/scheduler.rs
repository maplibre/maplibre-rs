use std::future::Future;

use crate::{error::Error, io::scheduler::Scheduler};

/// Multi-threading with Tokio.
pub struct TokioScheduler;

impl TokioScheduler {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scheduler for TokioScheduler {
    fn schedule<T>(&self, future_factory: impl FnOnce() -> T + Send + 'static) -> Result<(), Error>
    where
        T: Future<Output = ()> + Send + 'static,
    {
        tokio::task::spawn((future_factory)());
        Ok(())
    }
}
