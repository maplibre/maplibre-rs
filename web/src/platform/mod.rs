use maplibre::error::Error;
use maplibre::io::scheduler::Scheduler;
use std::future::Future;

pub mod http_client;

#[cfg(target_feature = "atomics")]
pub mod sync;

pub mod unsync;

pub struct NopScheduler;

impl Scheduler for NopScheduler {
    fn schedule<T>(&self, future_factory: impl FnOnce() -> T + Send + 'static) -> Result<(), Error>
    where
        T: Future<Output = ()> + 'static,
    {
        Err(Error::Schedule)
    }
}
