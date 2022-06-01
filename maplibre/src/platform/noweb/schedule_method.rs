use crate::error::Error;
use crate::ScheduleMethod;
use std::future::Future;
use tokio::task;
use tokio_util::task::LocalPoolHandle;

/// Multi-threading with Tokio.
pub struct TokioScheduleMethod {
    pool: LocalPoolHandle,
}

impl TokioScheduleMethod {
    pub fn new() -> Self {
        Self {
            pool: LocalPoolHandle::new(4),
        }
    }
}

impl ScheduleMethod for TokioScheduleMethod {
    fn schedule<T>(&self, future_factory: impl FnOnce() -> T + Send + 'static) -> Result<(), Error>
    where
        T: Future<Output = ()> + 'static,
    {
        self.pool.spawn_pinned(|| {
            let unsend_data = (future_factory)();

            async move { unsend_data.await }
        });

        Ok(())
    }
}
