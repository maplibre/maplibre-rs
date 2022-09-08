//! A small module that's intended to provide an example of creating a pool of
//! web workers which can be used to execute work.
//! Adopted from [wasm-bindgen example](https://github.com/rustwasm/wasm-bindgen/blob/0eba2efe45801b71f8873bc368c58a8ed8e894ff/examples/raytrace-parallel/src/pool.rs)

use std::{cell::RefCell, rc::Rc};

use js_sys::Promise;
use rand::prelude::*;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{DedicatedWorkerGlobalScope, ErrorEvent, Event, MessageEvent, Worker};

#[wasm_bindgen()]
extern "C" {
    #[wasm_bindgen(js_name = newWorker)]
    fn new_worker() -> JsValue;
}

pub struct WorkerPool {
    new_worker: Box<dyn Fn() -> Worker>,
    state: Rc<PoolState>,
}

struct PoolState {
    workers: RefCell<Vec<Worker>>,
    callback: Closure<dyn FnMut(Event)>,
}

struct Work {
    func: Box<dyn (FnOnce() -> Promise) + Send>,
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
    pub fn new(initial: usize, new_worker: Box<dyn Fn() -> Worker>) -> Result<WorkerPool, JsValue> {
        let pool = WorkerPool {
            new_worker,
            state: Rc::new(PoolState {
                workers: RefCell::new(Vec::with_capacity(initial)),
                callback: Closure::wrap(Box::new(|event: Event| {
                    log::error!("unhandled event: {}", event.type_());
                }) as Box<dyn FnMut(Event)>),
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
    fn spawn(&self) -> Result<(), JsValue> {
        log::info!("spawning new worker");
        let worker = (self.new_worker)();

        // With a worker spun up send it the module/memory so it can start
        // instantiating the wasm module. Later it might receive further
        // messages about code to run on the wasm module.
        let array = js_sys::Array::new();
        array.push(&wasm_bindgen::module());
        array.push(&wasm_bindgen::memory());
        worker.post_message(&array)?;

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
    fn worker(&self) -> Result<Worker, JsValue> {
        let workers = self.state.workers.borrow();
        let result = match workers.choose(&mut rand::thread_rng()) {
            Some(worker) => Some(worker),
            None => None,
        };

        if result.is_none() {
            self.spawn();
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
    pub fn execute(&self, f: impl (FnOnce() -> Promise) + Send + 'static) -> Result<(), JsValue> {
        let worker = self.worker()?;
        let work = Box::new(Work { func: Box::new(f) });
        let ptr = Box::into_raw(work);
        match worker.post_message(&JsValue::from(ptr as u32)) {
            Ok(()) => Ok(()),
            Err(e) => {
                unsafe {
                    drop(Box::from_raw(ptr));
                }
                Err(e)
            }
        }
    }
}

impl PoolState {
    fn push(&self, worker: Worker) {
        //worker.set_onmessage(Some(self.callback.as_ref().unchecked_ref()));
        //worker.set_onerror(Some(self.callback.as_ref().unchecked_ref()));
        let mut workers = self.workers.borrow_mut();
        for existing_worker in workers.iter() {
            assert!(existing_worker as &JsValue != &worker as &JsValue);
        }
        workers.push(worker);
    }
}

/// Entry point invoked by `worker.js`, a bit of a hack but see the "TODO" above
/// about `worker.js` in general.
#[wasm_bindgen]
pub async fn child_entry_point(ptr: u32) -> Result<(), JsValue> {
    let ptr = unsafe { Box::from_raw(ptr as *mut Work) };
    //let global = js_sys::global().unchecked_into::<DedicatedWorkerGlobalScope>();
    JsFuture::from((ptr.func)()).await?;
    //global.post_message(&JsValue::undefined())?;
    Ok(())
}
