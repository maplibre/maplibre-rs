use std::future::Future;

use maplibre::{error::Error, io::scheduler::ScheduleMethod};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::Worker;

use super::pool::WorkerPool;

pub struct WebWorkerPoolScheduleMethod {
    pool: WorkerPool,
}

impl WebWorkerPoolScheduleMethod {
    pub fn new(new_worker: js_sys::Function) -> Self {
        Self {
            pool: WorkerPool::new(
                4,
                Box::new(move || {
                    new_worker
                        .call0(&JsValue::undefined())
                        .unwrap()
                        .dyn_into::<Worker>()
                        .unwrap()
                }),
            )
            .unwrap(),
        }
    }
}

impl ScheduleMethod for WebWorkerPoolScheduleMethod {
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
            .unwrap();
        Ok(())
    }
}
