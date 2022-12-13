use std::{mem, rc::Rc};

use js_sys::ArrayBuffer;
use maplibre::{
    benchmarking::io::{
        apc::{AsyncProcedure, Input},
        source_client::{HttpSourceClient, SourceClient},
    },
    io::apc::CallError,
};
use thiserror::Error;
use wasm_bindgen::{prelude::*, JsCast};

use crate::{
    error::JSError,
    platform::singlethreaded::{
        apc::{MessageTag, ReceivedType},
        transferables::FlatBufferTransferable,
        PassingContext, UsedContext,
    },
    WHATWGFetchHttpClient,
};

/// Entry point invoked by the worker.
#[wasm_bindgen]
pub async fn singlethreaded_worker_entry(procedure_ptr: u32, input: String) -> Result<(), JSError> {
    let procedure: AsyncProcedure<UsedContext> = unsafe { mem::transmute(procedure_ptr) };

    let input =
        serde_json::from_str::<Input>(&input).map_err(|e| CallError::Deserialize(Box::new(e)))?;

    let context = PassingContext {
        source_client: SourceClient::new(HttpSourceClient::new(WHATWGFetchHttpClient::new())),
    };

    procedure(input, context).await?;

    Ok(())
}

#[derive(Error, Debug)]
#[error("unable to deserialize message sent by postMessage()")]
pub struct DeserializeMessage;

/// Entry point invoked by the main thread.
#[wasm_bindgen]
pub unsafe fn singlethreaded_main_entry(
    received_ptr: *const ReceivedType,
    in_transfer: js_sys::Array,
) -> Result<(), JSError> {
    let tag = in_transfer
        .get(0)
        .as_f64()
        .ok_or_else(|| CallError::Deserialize(Box::new(DeserializeMessage)))? as u32; // TODO: Is this cast fine?
    let buffer: ArrayBuffer = in_transfer
        .get(1)
        .dyn_into()
        .map_err(|_e| CallError::Deserialize(Box::new(DeserializeMessage)))?;

    let tag = MessageTag::from_u32(tag).map_err(|e| CallError::Deserialize(Box::new(e)))?;

    let message = tag.create_message(FlatBufferTransferable::from_array_buffer(buffer));

    // FIXME: Can we make this call safe? check if it was cloned before?
    let received: Rc<ReceivedType> = Rc::from_raw(received_ptr);

    // MAJOR FIXME: Fix mutability
    received
        .try_borrow_mut()
        .expect("Failed to borrow in singlethreaded_main_entry")
        .push(message);

    mem::forget(received); // FIXME: Enforce this somehow

    Ok(())
}
