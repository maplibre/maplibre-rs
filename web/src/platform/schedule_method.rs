use futures::executor::LocalPool;
use futures::task::{LocalSpawnExt, SpawnExt};
use log::warn;
use std::future::Future;

use super::pool::WorkerPool;
use maplibre::{error::Error, io::scheduler::ScheduleMethod};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::Worker;

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
                pool.spawner()
                    .spawn_local(async move {
                        future_factory().await;
                    })
                    .unwrap();

                warn!("Running tasks");
                pool.run_until_stalled();
                warn!("All tasks done");
            })
            .unwrap();
        Ok(())
    }
}
