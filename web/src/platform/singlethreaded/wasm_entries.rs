use std::{mem, rc::Rc};

use js_sys::{ArrayBuffer, Uint8Array};
use log::{error, info};
use maplibre::{
    benchmarking::io::{
        apc::{AsyncProcedure, Input, Message},
        source_client::{HttpSourceClient, SourceClient},
    },
    io::transferables::Transferables,
};
use wasm_bindgen::{prelude::*, JsCast};

use crate::{
    platform::singlethreaded::{
        apc::{ReceivedType, SerializedMessageTag},
        transferables::FlatBufferTransferable,
        PassingContext, UsedContext, UsedTransferables,
    },
    WHATWGFetchHttpClient,
};

/// Entry point invoked by the worker.
#[wasm_bindgen]
pub async fn singlethreaded_worker_entry(procedure_ptr: u32, input: String) -> Result<(), JsValue> {
    let procedure: AsyncProcedure<UsedContext> = unsafe { mem::transmute(procedure_ptr) };

    let input = serde_json::from_str::<Input>(&input).unwrap(); // FIXME (wasm-executor): Remove unwrap

    let context = PassingContext {
        source_client: SourceClient::new(HttpSourceClient::new(WHATWGFetchHttpClient::new())),
    };

    let result = (procedure)(input, context).await;

    if let Err(e) = result {
        error!("{:?}", e); // TODO handle better
    }

    Ok(())
}

/// Entry point invoked by the main thread.
#[wasm_bindgen]
pub unsafe fn singlethreaded_main_entry(
    received_ptr: *const ReceivedType,
    in_transfer_obj: js_sys::Array,
) -> Result<(), JsValue> {
    // FIXME (wasm-executor): Can we make this call safe? check if it was cloned before?
    let received: Rc<ReceivedType> = Rc::from_raw(received_ptr);

    let tag = in_transfer_obj.get(0).as_f64().unwrap() as u32;
    let tag = SerializedMessageTag::from_u32(tag).unwrap();

    info!("singlethreaded_main_entry {:?}", tag);

    let buffer: ArrayBuffer = in_transfer_obj.get(1).dyn_into().unwrap();
    let buffer = Uint8Array::new(&buffer);

    type TileTessellated = <UsedTransferables as Transferables>::TileTessellated;
    type UnavailableLayer = <UsedTransferables as Transferables>::LayerUnavailable;
    type IndexedLayer = <UsedTransferables as Transferables>::LayerIndexed;

    let transferable = FlatBufferTransferable {
        data: buffer.to_vec(),
        start: 0,
    };

    // TODO: Verify that data matches tag

    let message = match tag {
        SerializedMessageTag::TileTessellated => {
            Message::<UsedTransferables>::TileTessellated(transferable)
        }
        SerializedMessageTag::LayerUnavailable => {
            Message::<UsedTransferables>::LayerUnavailable(transferable)
        }
        SerializedMessageTag::LayerTessellated => {
            Message::<UsedTransferables>::LayerTessellated(transferable)
        }
        SerializedMessageTag::LayerIndexed => {
            Message::<UsedTransferables>::LayerIndexed(transferable)
        }
    };

    // MAJOR FIXME: Fix mutability
    received
        .try_borrow_mut()
        .expect("Failed to borrow in singlethreaded_main_entry")
        .push(message);

    mem::forget(received); // FIXME (wasm-executor): Enforce this somehow

    Ok(())
}
