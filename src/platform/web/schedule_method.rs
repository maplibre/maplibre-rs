use std::thread::Thread;

use log::warn;

use js_sys::{ArrayBuffer, Error as JSError, Uint8Array};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::Worker;
use web_sys::{Request, RequestInit, RequestMode, Response, WorkerGlobalScope};

use crate::coords::{TileCoords, WorldTileCoords};
use crate::error::Error;
use crate::io::scheduler::{IOScheduler, ScheduleMethod, ThreadLocalTessellatorState};
use crate::io::tile_cache::TileCache;
use crate::io::TileRequestID;

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

    pub fn schedule<T>(&self, future_factory: impl (FnOnce() -> T) + Send + 'static)
    where
        T: std::future::Future + 'static,
        T::Output: Send + 'static,
    {
        self.pool
            .run(move || {
                wasm_bindgen_futures::future_to_promise(async move {
                    future_factory().await;
                    Ok(JsValue::undefined())
                })
            })
            .unwrap();
    }
}
