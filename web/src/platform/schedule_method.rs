use std::future::Future;
use std::thread::Thread;

use js_sys::{ArrayBuffer, Error as JSError, Uint8Array};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::Worker;
use web_sys::{Request, RequestInit, RequestMode, Response, WorkerGlobalScope};

use maplibre::coords::{TileCoords, WorldTileCoords};
use maplibre::error::Error;
use maplibre::io::scheduler::{ScheduleMethod, Scheduler};
use maplibre::io::shared_thread_state::SharedThreadState;
use maplibre::io::tile_cache::TileCache;
use maplibre::io::TileRequestID;

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

impl Default for WebWorkerPoolScheduleMethod {
    fn default() -> Self {
        todo!()
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
