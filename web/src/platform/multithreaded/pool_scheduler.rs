use std::future::Future;

use maplibre::{benchmarking::io::scheduler::ScheduleError, io::scheduler::Scheduler};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::Worker;

use super::pool::WorkerPool;
use crate::error::WebError;

pub struct WebWorkerPoolScheduler {
    pool: WorkerPool,
}

impl WebWorkerPoolScheduler {
    pub fn new(new_worker: js_sys::Function) -> Result<Self, WebError> {
        let pool = WorkerPool::new(
            1,
            Box::new(move || {
                new_worker
                    .call0(&JsValue::undefined())
                    .map_err(WebError::from)?
                    .dyn_into::<Worker>()
                    .map_err(|_e| WebError::TypeError("Unable to cast to Worker".into()))
            }),
        )?;
        Ok(Self { pool })
    }
}

impl Scheduler for WebWorkerPoolScheduler {
    fn schedule<T>(
        &self,
        future_factory: impl (FnOnce() -> T) + Send + 'static,
    ) -> Result<(), ScheduleError>
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
            .map_err(|e| ScheduleError::Scheduling(Box::new(e)))
    }
}
