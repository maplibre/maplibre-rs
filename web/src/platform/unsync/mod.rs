use js_sys::{ArrayBuffer, Uint8Array};
use maplibre::error::Error;
use maplibre::io::scheduler::Scheduler;
use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::future::Future;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{DedicatedWorkerGlobalScope, Worker};

pub struct UnsyncScheduler {
    new_worker: Box<dyn Fn() -> Worker>,
    workers: RefCell<Vec<Worker>>,
}

impl UnsyncScheduler {
    pub fn new(new_worker: js_sys::Function) -> Self {
        let create_new_worker = Box::new(move || {
            new_worker
                .call0(&JsValue::undefined())
                .unwrap()
                .dyn_into::<Worker>()
                .unwrap()
        });

        let worker = create_new_worker();

        let array = js_sys::Array::new();
        array.push(&wasm_bindgen::module());
        worker.post_message(&array).unwrap();

        log::info!("new unsync");

        Self {
            new_worker: create_new_worker,
            workers: RefCell::new(vec![worker]),
        }
    }
}

impl Scheduler for UnsyncScheduler {
    fn schedule<T>(&self, future_factory: impl FnOnce() -> T + Send + 'static) -> Result<(), Error>
    where
        T: Future<Output = ()> + 'static,
    {
        self.workers.borrow()[0]
            .post_message(&JsValue::undefined())
            .unwrap();

        Ok(())
    }
}

thread_local! {
    static VEC: RefCell<Vec<u8>> = RefCell::new(vec![165u8, 162, 145, 224, 111]);
}

/// Entry point invoked by the worker.
#[wasm_bindgen]
pub async fn worker_entry() -> Result<(), JsValue> {
    log::info!("worker_entry unsync");

    //let vec = vec![165u8, 162, 145, 224, 111];

    VEC.with(|d| {
        let mut ref_mut = d.borrow_mut();
        ref_mut[0] += 1;

        unsafe {
            let uint8 = Uint8Array::view(&ref_mut);

            let array_buffer = uint8.buffer();

            log::info!(
                "{}",
                array_buffer
                    == wasm_bindgen::memory()
                        .dyn_into::<js_sys::WebAssembly::Memory>()
                        .unwrap()
                        .buffer()
                        .dyn_into::<ArrayBuffer>()
                        .unwrap()
            );

            let global = js_sys::global().unchecked_into::<DedicatedWorkerGlobalScope>();
            let array = js_sys::Array::new();
            array.push(&array_buffer);
            array.push(&JsValue::from(uint8.byte_offset()));
            global.post_message(&array).unwrap();
        };
    });

    Ok(())
}
