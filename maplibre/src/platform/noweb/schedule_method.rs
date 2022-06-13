use crate::error::Error;
use crate::ScheduleMethod;
use std::future::Future;

/// Multi-threading with Tokio.
pub struct TokioScheduleMethod;

impl TokioScheduleMethod {
    pub fn new() -> Self {
        Self {}
    }
}

impl ScheduleMethod for TokioScheduleMethod {
    fn schedule<T>(&self, future_factory: impl FnOnce() -> T + Send + 'static) -> Result<(), Error>
    where
        T: Future<Output = ()> + Send + 'static,
    {
        tokio::task::spawn((future_factory)());
        Ok(())
    }
}
