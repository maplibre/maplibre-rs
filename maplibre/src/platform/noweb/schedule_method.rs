use crate::error::Error;
use crate::io::shared_thread_state::SharedThreadState;
use crate::ScheduleMethod;
use std::future::Future;
use std::pin::Pin;

/// Multi-threading with Tokio.
pub struct TokioScheduleMethod;

impl TokioScheduleMethod {
    pub fn new() -> Self {
        Self {}
    }
}

impl ScheduleMethod for TokioScheduleMethod {
    fn schedule(
        &self,
        shared_thread_state: SharedThreadState,
        future_factory: Box<
            (dyn (FnOnce(SharedThreadState) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>)
                 + Send
                 + 'static),
        >,
    ) -> Result<(), Error> {
        tokio::task::spawn((future_factory)(shared_thread_state));
        Ok(())
    }
}
