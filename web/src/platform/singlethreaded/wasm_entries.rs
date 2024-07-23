use std::{mem, rc::Rc, sync::OnceLock};

use log::error;
use maplibre::{
    benchmarking::io::{
        apc::{AsyncProcedure, Input, Message},
        source_client::{HttpSourceClient, SourceClient},
    },
    environment::OffscreenKernel,
    io::apc::CallError,
};
use thiserror::Error;
use wasm_bindgen::prelude::*;
use web_sys::DedicatedWorkerGlobalScope;

use crate::{
    error::JSError,
    platform::{
        singlethreaded::{
            apc::{ReceivedType, WebMessageTag},
            transferables::FlatBufferTransferable,
            PassingContext, UsedContext,
        },
        UsedOffscreenKernelEnvironment,
    },
    WHATWGFetchHttpClient,
};

static CONFIG: OnceLock<String> = OnceLock::new();

fn kernel_config() -> &'static str {
    CONFIG.get().map(move |t| t.as_str()).unwrap_or("{}")
}

#[wasm_bindgen]
pub fn set_kernel_config(config: String) {
    CONFIG.set(config).expect("failed to set kernel config")
}

/// Entry point invoked by the worker. Processes data and sends the result back to the main thread.
#[wasm_bindgen]
pub async fn singlethreaded_process_data(procedure_ptr: u32, input: String) -> Result<(), JSError> {
    let procedure: AsyncProcedure<UsedOffscreenKernelEnvironment, UsedContext> =
        unsafe { mem::transmute(procedure_ptr) };

    let input = serde_json::from_str::<Input>(&input).map_err(|e| {
        CallError::DeserializeInput(Box::new(e)) // TODO: This error e is not logged
    })?;

    let context = PassingContext {
        source_client: SourceClient::new(HttpSourceClient::new(WHATWGFetchHttpClient::default())),
    };

    if let Ok(global) = js_sys::global().dyn_into::<DedicatedWorkerGlobalScope>() {
        let name = global.name();
        log::info!(
            "Processing on web worker: {}",
            if name.is_empty() {
                "name not set"
            } else {
                name.as_str()
            }
        );
    }

    procedure(
        input,
        context,
        UsedOffscreenKernelEnvironment::create(serde_json::from_str(&kernel_config()).unwrap()),
    )
    .await?; // TODO

    Ok(())
}

#[derive(Error, Debug)]
#[error("unable to deserialize message sent by postMessage()")]
pub struct DeserializeMessage;

/// Entry point invoked by the main thread. Receives data on the main thread and makes it available
/// to the renderer.
#[wasm_bindgen]
pub fn singlethreaded_receive_data(
    received_ptr: *const ReceivedType,
    tag: u32,
    buffer: js_sys::ArrayBuffer,
) -> Result<(), JSError> {
    let tag = WebMessageTag::from_u32(tag).map_err(|e| CallError::Deserialize(Box::new(e)))?;

    log::debug!(
        "received message ({tag:?}) with {}bytes on main thread",
        buffer.byte_length()
    );

    let message = Message::new(
        tag.to_static(),
        Box::new(FlatBufferTransferable::from_array_buffer(tag, buffer)),
    );

    // FIXME: Can we make this call safe? check if it was cloned before?
    let received: Rc<ReceivedType> = unsafe { Rc::from_raw(received_ptr) };

    // MAJOR FIXME: Fix mutability
    received
        .try_borrow_mut()
        .expect("Failed to borrow in singlethreaded_main_entry")
        .push(message);

    mem::forget(received); // FIXME: Enforce this somehow

    Ok(())
}
