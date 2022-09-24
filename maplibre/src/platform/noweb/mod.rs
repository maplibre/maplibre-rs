//! Module which is used target platform is not web related.

use std::future::Future;

pub mod http_client;
pub mod scheduler;
pub mod trace;

pub fn run_multithreaded<F: Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_io()
        .enable_time()
        .on_thread_start(|| {
            #[cfg(feature = "trace")]
            tracy_client::set_thread_name!("tokio-runtime-worker");
        })
        .build()
        .unwrap()
        .block_on(future)
}
