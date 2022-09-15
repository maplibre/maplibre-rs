use maplibre::error::Error;
use std::future::Future;
pub mod http_client;

#[cfg(target_feature = "atomics")]
pub mod sync;
#[cfg(target_feature = "atomics")]
type Scheduler = sync::pool_scheduler::WebWorkerPoolScheduler;
#[cfg(target_feature = "atomics")]
pub type AsyncProcedureCall = sync::apc::AtomicAsyncProcedureCall;

#[cfg(not(target_feature = "atomics"))]
pub mod unsync;
#[cfg(not(target_feature = "atomics"))]
pub type AsyncProcedureCall = unsync::apc::PassingAsyncProcedureCall;
