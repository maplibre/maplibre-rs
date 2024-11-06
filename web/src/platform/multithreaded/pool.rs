//! A small module that's intended to provide an example of creating a pool of
//! web workers which can be used to execute work.
//! Adopted from [wasm-bindgen example](https://github.com/rustwasm/wasm-bindgen/blob/0eba2efe45801b71f8873bc368c58a8ed8e894ff/examples/raytrace-parallel/src/pool.rs)

use std::{cell::RefCell, rc::Rc};

use rand::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::Worker;

use crate::error::WebError;

#[wasm_bindgen()]
extern "C" {
    #[wasm_bindgen(js_name = newWorker)]
    fn new_worker() -> JsValue;
}

pub type PinnedFuture = std::pin::Pin<Box<(dyn std::future::Future<Output = ()> + 'static)>>;

type NewWorker = Box<dyn Fn() -> Result<Worker, WebError>>;
type Execute = Box<dyn (FnOnce() -> PinnedFuture) + Send>;

pub struct WorkerPool {
    new_worker: NewWorker,
    state: Rc<PoolState>,
}

struct PoolState {
    workers: RefCell<Vec<Worker>>,
}

impl PoolState {
    fn push(&self, worker: Worker) {
        let mut workers = self.workers.borrow_mut();
        for existing_worker in workers.iter() {
            assert_ne!(existing_worker as &JsValue, &worker as &JsValue);
        }
        workers.push(worker);
    }
}

pub struct Work {
    func: Execute,
}

impl Work {
    pub fn execute(self) -> PinnedFuture {
        (self.func)()
    }
}

impl WorkerPool {
    /// Creates a new `WorkerPool` which immediately creates `initial` workers.
    ///
    /// The pool created here can be used over a long period of time, and it
    /// will be initially primed with `initial` workers. Currently workers are
    /// never released or gc'd until the whole pool is destroyed.
    ///
    /// # Errors
    ///
    /// Returns any error that may happen while a JS web worker is created and a
    /// message is sent to it.
    pub fn new(initial: usize, new_worker: NewWorker) -> Result<WorkerPool, WebError> {
        let pool = WorkerPool {
            new_worker,
            state: Rc::new(PoolState {
                workers: RefCell::new(Vec::with_capacity(initial)),
            }),
        };
        for _ in 0..initial {
            pool.spawn()?;
        }

        Ok(pool)
    }

    /// Unconditionally spawns a new worker.
    ///
    /// The worker isn't registered with this `WorkerPool` but is capable of
    /// executing work for this wasm module.
    ///
    /// # Errors
    ///
    /// Returns any error that may happen while a JS web worker is created and a
    /// message is sent to it.
    fn spawn(&self) -> Result<(), WebError> {
        log::info!("spawning new worker");
        let worker = (self.new_worker)()?;

        // With a worker spun up send it the module/memory so it can start
        // instantiating the wasm module. Later it might receive further
        // messages about code to run on the wasm module.
        worker.post_message(
            &js_sys::Object::from_entries(&js_sys::Array::of3(
                &js_sys::Array::of2(&JsValue::from("type"), &js_sys::JsString::from("wasm_init")),
                &js_sys::Array::of2(&JsValue::from("module"), &wasm_bindgen::module()),
                &js_sys::Array::of2(&JsValue::from("memory"), &wasm_bindgen::memory()),
            ))
            .expect("can not fail"),
        )?;

        self.state.push(worker);
        Ok(())
    }

    /// Fetches a worker from this pool, spawning one if necessary.
    ///
    /// This will attempt to pull an already-spawned web worker from our cache
    /// if one is available, otherwise it will spawn a new worker and return the
    /// newly spawned worker.
    ///
    /// # Errors
    ///
    /// Returns any error that may happen while a JS web worker is created and a
    /// message is sent to it.
    fn worker(&self) -> Result<Worker, WebError> {
        let workers = self.state.workers.borrow();
        let result = workers.choose(&mut thread_rng());

        if result.is_none() {
            self.spawn()?;
        }

        match result {
            Some(worker) => Ok(worker.clone()),
            None => self.worker(),
        }
    }

    /// Executes the work `f` in a web worker, spawning a web worker if
    /// necessary.
    ///
    /// This will acquire a web worker and then send the closure `f` to the
    /// worker to execute. The worker won't be usable for anything else while
    /// `f` is executing, and no callbacks are registered for when the worker
    /// finishes.
    ///
    /// # Errors
    ///
    /// Returns any error that may happen while a JS web worker is created and a
    /// message is sent to it.
    pub fn execute(
        &self,
        f: impl (FnOnce() -> PinnedFuture) + Send + 'static,
    ) -> Result<(), WebError> {
        let worker = self.worker()?;
        let work = Work { func: Box::new(f) };
        let work_ptr = Box::into_raw(Box::new(work));
        match worker.post_message(
            &js_sys::Object::from_entries(&js_sys::Array::of2(
                &js_sys::Array::of2(&JsValue::from("type"), &js_sys::JsString::from("pool_call")),
                &js_sys::Array::of2(&JsValue::from("work_ptr"), &JsValue::from(work_ptr as u32)),
            ))
            .expect("can not fail"),
        ) {
            Ok(()) => Ok(()),
            Err(e) => {
                unsafe {
                    drop(Box::from_raw(work_ptr));
                }
                Err(e.into())
            }
        }
    }
}
