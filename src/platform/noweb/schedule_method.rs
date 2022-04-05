use crate::io::scheduler::{Scheduler, ThreadLocalState};

pub struct TokioScheduleMethod;

impl TokioScheduleMethod {
    pub fn new() -> Self {
        Self {}
    }

    pub fn schedule<T>(
        &self,
        scheduler: &Scheduler,
        future_factory: impl (FnOnce(ThreadLocalState) -> T) + Send + 'static,
    ) where
        T: std::future::Future + Send + 'static,
        T::Output: Send + 'static,
    {
        tokio::task::spawn(future_factory(scheduler.new_tessellator_state()));
    }
}
