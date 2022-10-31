use std::future::Future;

use log::warn;
use maplibre::{error::Error, io::scheduler::Scheduler};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::Worker;

use super::pool::WorkerPool;
use crate::CurrentEnvironment;

pub struct WebWorkerPoolScheduler {
    pool: WorkerPool,
}

impl WebWorkerPoolScheduler {
    pub fn new(new_worker: js_sys::Function) -> Self {
        // TODO: Are expects here oke?
        let pool = WorkerPool::new(
            1,
            Box::new(move || {
                new_worker
                    .call0(&JsValue::undefined())
                    .expect("Unable to call new_worker function")
                    .dyn_into::<Worker>()
                    .expect("new_worker function did not return a Worker")
            }),
        )
        .expect("Unable to create WorkerPool");
        Self { pool }
    }
}

impl Scheduler for WebWorkerPoolScheduler {
    fn schedule<T>(
        &self,
        future_factory: impl (FnOnce() -> T) + Send + 'static,
    ) -> Result<(), Error>
    where
        T: Future<Output = ()> + 'static,
    {
        self.pool
            .execute(move || {
                wasm_bindgen_futures::future_to_promise(async move {
                    future_factory().await;
                    Ok(JsValue::undefined())
                })
            })
            .unwrap(); // FIXME (wasm-executor): remove unwrap
        Ok(())
    }
}
