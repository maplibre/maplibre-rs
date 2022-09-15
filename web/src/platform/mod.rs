use maplibre::error::Error;
use std::future::Future;
pub mod http_client;

#[cfg(target_feature = "atomics")]
pub mod sync;

#[cfg(not(target_feature = "atomics"))]
pub mod unsync;
