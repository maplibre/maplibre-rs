use crate::error::Error;
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
        future_factory: Box<
            (dyn (FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>) + Send + 'static),
        >,
    ) -> Result<(), Error> {
        tokio::task::spawn((future_factory)());
        Ok(())
    }
}
