use std::future::Future;

use crate::error::Error;

use crate::io::shared_thread_state::SharedThreadState;

pub struct Scheduler<SM: ScheduleMethod> {
    schedule_method: SM,
}

impl<SM: ScheduleMethod> Scheduler<SM> {
    pub fn new(schedule_method: SM) -> Self {
        Self { schedule_method }
    }

    pub fn schedule_method(&self) -> &SM {
        &self.schedule_method
    }
}

pub trait ScheduleMethod: Default + 'static {
    #[cfg(not(feature = "no-thread-safe-futures"))]
    fn schedule<T>(
        &self,
        shared_thread_state: SharedThreadState,
        future_factory: impl (FnOnce(SharedThreadState) -> T) + Send + 'static,
    ) -> Result<(), Error>
    where
        T: Future<Output = ()> + Send + 'static;

    #[cfg(feature = "no-thread-safe-futures")]
    fn schedule<T>(
        &self,
        shared_thread_state: SharedThreadState,
        future_factory: impl (FnOnce(SharedThreadState) -> T) + Send + 'static,
    ) -> Result<(), Error>
    where
        T: Future<Output = ()> + 'static;
}
