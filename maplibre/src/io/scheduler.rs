//! Scheduling.

use std::future::Future;

use crate::error::Error;

/// Async/await scheduler.
pub struct Scheduler<SM>
where
    SM: ScheduleMethod,
{
    schedule_method: SM,
}

impl<SM> Scheduler<SM>
where
    SM: ScheduleMethod,
{
    pub fn new(schedule_method: SM) -> Self {
        Self { schedule_method }
    }

    pub fn schedule_method(&self) -> &SM {
        &self.schedule_method
    }
}

/// Can schedule a task from a future factory and a shared state.
pub trait ScheduleMethod: 'static {
    fn schedule<T>(
        &self,
        future_factory: impl (FnOnce() -> T) + Send + 'static,
    ) -> Result<(), Error>
    where
        T: Future<Output = ()> + 'static;
}
