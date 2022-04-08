
use crate::io::shared_thread_state::SharedThreadState;

pub struct TokioScheduleMethod;

impl TokioScheduleMethod {
    pub fn new() -> Self {
        Self {}
    }

    pub fn schedule<T>(
        &self,
        shared_thread_state: SharedThreadState,
        future_factory: impl (FnOnce(SharedThreadState) -> T) + Send + 'static,
    ) where
        T: std::future::Future + Send + 'static,
        T::Output: Send + 'static,
    {
        tokio::task::spawn(future_factory(shared_thread_state));
    }
}
