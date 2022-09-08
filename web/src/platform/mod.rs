use maplibre::error::Error;
use maplibre::io::scheduler::ScheduleMethod;
use std::future::Future;

pub mod http_client;

#[cfg(target_feature = "atomics")]
pub mod pool;
#[cfg(target_feature = "atomics")]
pub mod pool_schedule_method;

pub struct NopScheduleMethod;

impl ScheduleMethod for NopScheduleMethod {
    fn schedule<T>(&self, future_factory: impl FnOnce() -> T + Send + 'static) -> Result<(), Error>
    where
        T: Future<Output = ()> + 'static,
    {
        Err(Error::Schedule)
    }
}
