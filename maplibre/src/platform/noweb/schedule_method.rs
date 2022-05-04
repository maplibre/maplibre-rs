use crate::error::Error;
use crate::io::shared_thread_state::SharedThreadState;
use crate::ScheduleMethod;
use std::future::Future;
use tokio::task;
use tokio_util::task::LocalPoolHandle;
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
    fn schedule<T>(
        &self,
        shared_thread_state: SharedThreadState,
        future_factory: impl FnOnce(SharedThreadState) -> T + Send + 'static,
    ) -> Result<(), Error>
    where
        T: Future<Output = ()> + 'static,
    {
        self.pool.spawn_pinned(|| {
            let unsend_data = (future_factory)(shared_thread_state);

            async move { unsend_data.await }
        });

        Ok(())
    }
}
