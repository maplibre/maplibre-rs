//! Module which is used target platform is not web related.

use std::{
    future::Future,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{
    environment::OffscreenKernelEnvironment,
    io::source_client::{HttpSourceClient, SourceClient},
    platform::http_client::ReqwestHttpClient,
};

pub mod http_client;
pub mod scheduler;
pub mod trace;

pub fn run_multithreaded<F: Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .thread_name_fn(|| {
            static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
            let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
            format!("maplibre-rs-pool-{id}")
        })
        .on_thread_start(|| {
            #[cfg(feature = "trace")]
            tracing_tracy::client::set_thread_name!("tokio-runtime-worker");

            log::info!("Worker thread started")
        })
        .build()
        .unwrap()
        .block_on(future)
}

pub struct ReqwestOffscreenKernelEnvironment;

impl OffscreenKernelEnvironment for ReqwestOffscreenKernelEnvironment {
    type HttpClient = ReqwestHttpClient;

    fn create() -> Self {
        ReqwestOffscreenKernelEnvironment
    }

    fn source_client(&self) -> SourceClient<Self::HttpClient> {
        SourceClient::new(HttpSourceClient::new(ReqwestHttpClient::new(None)))
    }
}
