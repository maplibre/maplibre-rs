use std::future::Future;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_sys::Worker;

use maplibre::error::Error;
use maplibre::io::scheduler::ScheduleMethod;
use maplibre::io::shared_thread_state::SharedThreadState;

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
        shared_thread_state: SharedThreadState,
        future_factory: impl (FnOnce(SharedThreadState) -> T) + Send + 'static,
    ) -> Result<(), Error>
    where
        T: Future<Output = ()> + 'static,
    {
        self.pool
            .run(move || {
                wasm_bindgen_futures::future_to_promise(async move {
                    future_factory(shared_thread_state).await;
                    Ok(JsValue::undefined())
                })
            })
            .unwrap();
        Ok(())
    }
}
