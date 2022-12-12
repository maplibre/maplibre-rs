//! Scheduling.

use std::future::Future;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScheduleError {
    #[error("scheduling work failed")]
    Scheduling(Box<dyn std::error::Error>),
    #[error("scheduler is not implemented on this platform")]
    NotImplemented,
}

/// Async/await scheduler.
/// Can schedule a task from a future factory and a shared state.
pub trait Scheduler: 'static {
    #[cfg(feature = "thread-safe-futures")]
    fn schedule<T>(
        &self,
        future_factory: impl (FnOnce() -> T) + Send + 'static,
    ) -> Result<(), ScheduleError>
    where
        T: Future<Output = ()> + Send + 'static;

    #[cfg(not(feature = "thread-safe-futures"))]
    fn schedule<T>(
        &self,
        future_factory: impl (FnOnce() -> T) + Send + 'static,
    ) -> Result<(), ScheduleError>
    where
        T: Future<Output = ()> + 'static;
}

pub struct NopScheduler;

impl Scheduler for NopScheduler {
    fn schedule<T>(
        &self,
        _future_factory: impl FnOnce() -> T + Send + 'static,
    ) -> Result<(), ScheduleError>
    where
        T: Future<Output = ()> + 'static,
    {
        Err(ScheduleError::NotImplemented)
    }
}
