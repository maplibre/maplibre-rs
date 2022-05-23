//! Scheduling.

use std::future::Future;
use std::pin::Pin;

use crate::error::Error;

use crate::io::shared_thread_state::SharedThreadState;

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

    pub fn take(self) -> SM {
        self.schedule_method
    }
}

/// Can schedule a task from a future factory and a shared state.
// Should be object safe in order to be able to have a dyn object in MapContext
pub trait ScheduleMethod: 'static {
    #[cfg(not(feature = "no-thread-safe-futures"))]
    fn schedule(
        &self,
        shared_thread_state: SharedThreadState,
        future_factory: Box<
            (dyn (FnOnce(SharedThreadState) -> Pin<Box<dyn Future<Output = ()> + Send>>) + Send),
        >,
    ) -> Result<(), Error>;

    #[cfg(feature = "no-thread-safe-futures")]
    fn schedule(
        &self,
        shared_thread_state: SharedThreadState,
        future_factory: Box<
            (dyn (FnOnce(SharedThreadState) -> Pin<Box<dyn Future<Output = ()>>>) + Send),
        >,
    ) -> Result<(), Error>;
}
