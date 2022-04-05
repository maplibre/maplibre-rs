use crate::io::scheduler::{IOScheduler, ThreadLocalTessellatorState};

pub struct TokioScheduleMethod;

impl TokioScheduleMethod {
    pub fn new() -> Self {
        Self {}
    }

    pub fn schedule<T>(
        &self,
        scheduler: &IOScheduler,
        future_factory: impl (FnOnce(ThreadLocalTessellatorState) -> T) + Send + 'static,
    ) where
        T: std::future::Future + Send + 'static,
        T::Output: Send + 'static,
    {
        tokio::task::spawn(future_factory(scheduler.new_tessellator_state()));
    }
}
