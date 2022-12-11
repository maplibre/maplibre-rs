//! Scheduling.

use std::{
    fmt::{Display, Formatter},
    future::Future,
};

#[derive(Debug)]
pub enum ScheduleError {
    Scheduling(Box<dyn std::error::Error>),
    NotImplemented,
}

impl Display for ScheduleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ScheduleError {}

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
