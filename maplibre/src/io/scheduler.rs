//! Scheduling.

use std::future::Future;

use crate::error::Error;

/// Async/await scheduler.
/// Can schedule a task from a future factory and a shared state.
pub trait Scheduler: 'static {
    #[cfg(feature = "thread-safe-futures")]
    fn schedule<T>(
        &self,
        future_factory: impl (FnOnce() -> T) + Send + 'static,
    ) -> Result<(), Error>
    where
        T: Future<Output = ()> + Send + 'static;

    #[cfg(not(feature = "thread-safe-futures"))]
    fn schedule<T>(
        &self,
        future_factory: impl (FnOnce() -> T) + Send + 'static,
    ) -> Result<(), Error>
    where
        T: Future<Output = ()> + 'static;
}

pub struct NopScheduler;

impl Scheduler for NopScheduler {
    fn schedule<T>(&self, future_factory: impl FnOnce() -> T + Send + 'static) -> Result<(), Error>
    where
        T: Future<Output = ()> + 'static,
    {
        Err(Error::Scheduler)
    }
}
