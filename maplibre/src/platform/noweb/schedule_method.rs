use crate::error::Error;
use crate::io::shared_thread_state::SharedThreadState;
use crate::ScheduleMethod;
use std::future::Future;

pub struct TokioScheduleMethod;

impl TokioScheduleMethod {
    pub fn new() -> Self {
        Self {}
    }
}

impl ScheduleMethod for TokioScheduleMethod {
    fn schedule<T>(
        &self,
        shared_thread_state: SharedThreadState,
        future_factory: impl FnOnce(SharedThreadState) -> T + Send + 'static,
    ) -> Result<(), Error>
    where
        T: Future<Output = ()> + Send + 'static,
    {
        tokio::task::spawn(future_factory(shared_thread_state));
        Ok(())
    }
}
