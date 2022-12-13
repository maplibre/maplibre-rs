pub mod http_client;

#[cfg(target_feature = "atomics")]
pub mod multithreaded;

#[cfg(not(target_feature = "atomics"))]
pub mod singlethreaded;
