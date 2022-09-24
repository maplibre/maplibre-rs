use std::future::Future;

use log::warn;
use maplibre::{error::Error, io::scheduler::Scheduler};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::Worker;

use super::pool::WorkerPool;

pub struct WebWorkerPoolScheduler {
    pool: WorkerPool,
}

impl WebWorkerPoolScheduler {
    pub fn new(new_worker: js_sys::Function) -> Self {
        Self {
            pool: WorkerPool::new(
                1,
                Box::new(move || {
                    new_worker
                        .call0(&JsValue::undefined())
                        .unwrap() // FIXME (wasm-executor): Remove unwrap
                        .dyn_into::<Worker>()
                        .unwrap() // FIXME (wasm-executor): remove unwrap
                }),
            )
            .unwrap(), // FIXME (wasm-executor): Remove unwrap
        }
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
