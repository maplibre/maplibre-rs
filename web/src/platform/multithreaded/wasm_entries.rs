use maplibre::io::apc::CallError;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::{platform::multithreaded::pool::Work, JSError};

/// Entry point invoked by the worker.
#[wasm_bindgen]
pub async fn multithreaded_worker_entry(ptr: u32) -> Result<(), JSError> {
    let work = unsafe { Box::from_raw(ptr as *mut Work) };
    JsFuture::from(work.execute())
        .await
        .map_err(|_e| CallError::Schedule)?;
    Ok(())
}
