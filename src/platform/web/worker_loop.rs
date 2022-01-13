use crate::io::worker_loop::WorkerLoop;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn create_worker_loop() -> *mut WorkerLoop {
    let worker_loop = Box::new(WorkerLoop::new());
    let ptr = Box::into_raw(worker_loop);
    return ptr;
}

#[wasm_bindgen]
pub async fn run_worker_loop(worker_loop_ptr: *mut WorkerLoop) {
    let mut worker_loop: Box<WorkerLoop> = unsafe { Box::from_raw(worker_loop_ptr) };

    // Either call forget or the worker loop to keep it alive
    worker_loop.run_loop().await;
    std::mem::forget(worker_loop);
}
