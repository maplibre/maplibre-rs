use maplibre::io::apc::CallError;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::{platform::multithreaded::pool::Work, JSError};

/// Entry point invoked by the worker.
#[wasm_bindgen]
pub async fn multithreaded_process_data(work_ptr: *mut Work) -> Result<(), JSError> {
    let work = unsafe { Box::from_raw(work_ptr) };
    JsFuture::from(work.execute())
        .await
        .map_err(|_e| CallError::Schedule)?;
    Ok(())
}
