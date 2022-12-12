use std::future::Future;

use crate::io::scheduler::{ScheduleError, Scheduler};

/// Multi-threading with Tokio.
pub struct TokioScheduler;

impl TokioScheduler {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scheduler for TokioScheduler {
    #[cfg(feature = "thread-safe-futures")]
    fn schedule<T>(
        &self,
        future_factory: impl FnOnce() -> T + Send + 'static,
    ) -> Result<(), ScheduleError>
    where
        T: Future<Output = ()> + Send + 'static,
    {
        tokio::task::spawn((future_factory)());
        Ok(())
    }

    // FIXME: Provide a working implementation
    #[cfg(not(feature = "thread-safe-futures"))]
    fn schedule<T>(
        &self,
        _future_factory: impl FnOnce() -> T + 'static,
    ) -> Result<(), ScheduleError>
    where
        T: Future<Output = ()> + 'static,
    {
        Ok(())
    }
}

impl Default for TokioScheduler {
    fn default() -> Self {
        Self::new()
    }
}
