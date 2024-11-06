use wasm_bindgen::prelude::*;

use crate::{platform::multithreaded::pool::Work, JSError};

/// Entry point invoked by the worker.
#[wasm_bindgen]
pub async fn multithreaded_process_data(work_ptr: *mut Work) -> Result<(), JSError> {
    let work: Box<Work> = unsafe { Box::from_raw(work_ptr) };
    work.execute().await;
    Ok(())
}
