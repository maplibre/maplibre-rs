use std::{cell::RefCell, mem, rc::Rc};

use log::info;
use maplibre::{
    error::Error,
    io::{
        apc::{AsyncProcedure, AsyncProcedureCall, Context, Input, Message},
        source_client::{HttpSourceClient, SourceClient},
        transferables::Transferables,
    },
};
use transferable_memory::{InTransferMemory, MemoryTransferable};
use wasm_bindgen::{prelude::*, JsCast, JsValue};
use web_sys::{DedicatedWorkerGlobalScope, Worker};

use crate::{
    platform::singlethreaded::transferables::{
        LargeTesselationData, LinearLayerIndexed, LinearLayerTessellated, LinearLayerUnavailable,
        LinearTileTessellated, LinearTransferables, VariableTessellationData,
    },
    WHATWGFetchHttpClient,
};

type UsedTransferables = LinearTransferables;
type UsedHttpClient = WHATWGFetchHttpClient;
type UsedContext = PassingContext;

#[derive(Debug)]
enum SerializedMessageTag {
    TileTessellated = 1,
    LayerUnavailable = 2,
    LayerTessellated = 3,
    LayerIndexed = 4,
}

impl SerializedMessageTag {
    fn from_u32(tag: u32) -> Option<Self> {
        match tag {
            x if x == SerializedMessageTag::LayerUnavailable as u32 => {
                Some(SerializedMessageTag::LayerUnavailable)
            }
            x if x == SerializedMessageTag::LayerTessellated as u32 => {
                Some(SerializedMessageTag::LayerTessellated)
            }
            x if x == SerializedMessageTag::TileTessellated as u32 => {
                Some(SerializedMessageTag::TileTessellated)
            }
            x if x == SerializedMessageTag::LayerIndexed as u32 => {
                Some(SerializedMessageTag::LayerIndexed)
            }
            _ => None,
        }
    }
}

trait SerializableMessage {
    fn serialize(&self) -> InTransferMemory;

    fn deserialize(
        tag: SerializedMessageTag,
        in_transfer: InTransferMemory,
    ) -> Message<UsedTransferables>;

    fn tag(&self) -> SerializedMessageTag;
}

impl SerializableMessage for Message<LinearTransferables> {
    fn serialize(&self) -> InTransferMemory {
        match self {
            Message::TileTessellated(message) => message.to_in_transfer(self.tag() as u32),
            Message::LayerUnavailable(message) => message.to_in_transfer(self.tag() as u32),
            Message::LayerTessellated(message) => match &message.data {
                VariableTessellationData::Large(message) => {
                    message.to_in_transfer(self.tag() as u32)
                }
            },
            Message::LayerIndexed(message) => message.to_in_transfer(self.tag() as u32),
        }
    }

    fn deserialize(
        tag: SerializedMessageTag,
        in_transfer: InTransferMemory,
    ) -> Message<UsedTransferables> {
        type TileTessellated = <UsedTransferables as Transferables>::TileTessellated;
        type UnavailableLayer = <UsedTransferables as Transferables>::LayerUnavailable;
        type IndexedLayer = <UsedTransferables as Transferables>::LayerIndexed;
        unsafe {
            match tag {
                SerializedMessageTag::TileTessellated => {
                    Message::<UsedTransferables>::TileTessellated(
                        LinearTileTessellated::from_in_transfer(in_transfer),
                    )
                }
                SerializedMessageTag::LayerUnavailable => {
                    Message::<UsedTransferables>::LayerUnavailable(
                        LinearLayerUnavailable::from_in_transfer(in_transfer),
                    )
                }
                SerializedMessageTag::LayerTessellated => {
                    Message::<UsedTransferables>::LayerTessellated(LinearLayerTessellated {
                        data: VariableTessellationData::Large(
                            LargeTesselationData::from_in_transfer_boxed(in_transfer),
                        ), // TODO DO not use only large
                    })
                }
                SerializedMessageTag::LayerIndexed => Message::<UsedTransferables>::LayerIndexed(
                    LinearLayerIndexed::from_in_transfer(in_transfer),
                ),
            }
        }
    }

    fn tag(&self) -> SerializedMessageTag {
        match self {
            Message::TileTessellated(_) => SerializedMessageTag::TileTessellated,
            Message::LayerUnavailable(_) => SerializedMessageTag::LayerUnavailable,
            Message::LayerTessellated(_) => SerializedMessageTag::LayerTessellated,
            Message::LayerIndexed(_) => SerializedMessageTag::LayerIndexed,
        }
    }
}

#[derive(Clone)]
pub struct PassingContext {
    source_client: SourceClient<UsedHttpClient>,
}

impl Context<UsedTransferables, UsedHttpClient> for PassingContext {
    fn send(&self, data: Message<UsedTransferables>) -> Result<(), Error> {
        let in_transfer = data.serialize();

        let global: DedicatedWorkerGlobalScope =
            js_sys::global().dyn_into().map_err(|_e| Error::APC)?;

        // TODO use object
        let transfer_obj = js_sys::Array::new();
        transfer_obj.push(&JsValue::from(in_transfer.type_id as u32));
        transfer_obj.push(&in_transfer.buffer);

        let transfer = js_sys::Array::new();
        transfer.push(&in_transfer.buffer);

        // TODO: Verify transfer
        global
            .post_message_with_transfer(&transfer_obj, &transfer)
            .map_err(|_e| Error::APC)
    }

    fn source_client(&self) -> &SourceClient<UsedHttpClient> {
        &self.source_client
    }
}

type ReceivedType = RefCell<Vec<Message<UsedTransferables>>>;

pub struct PassingAsyncProcedureCall {
    new_worker: Box<dyn Fn() -> Worker>,
    workers: Vec<Worker>,

    received: Rc<ReceivedType>, // FIXME (wasm-executor): Is RefCell fine?
}

impl PassingAsyncProcedureCall {
    pub fn new(new_worker: js_sys::Function, initial_workers: u8) -> Self {
        let received = Rc::new(RefCell::new(vec![]));
        let received_ref = received.clone();

        let create_new_worker = Box::new(move || {
            new_worker
                .call1(
                    &JsValue::undefined(),
                    &JsValue::from(Rc::into_raw(received_ref.clone()) as u32),
                )
                .unwrap() // FIXME (wasm-executor): Remove unwrap
                .dyn_into::<Worker>()
                .unwrap() // FIXME (wasm-executor): Remove unwrap
        });

        let workers = (0..initial_workers)
            .map(|_| {
                let worker: Worker = create_new_worker();

                let array = js_sys::Array::new();
                array.push(&wasm_bindgen::module());
                worker.post_message(&array).unwrap(); // FIXME (wasm-executor): Remove unwrap
                worker
            })
            .collect::<Vec<_>>();

        Self {
            new_worker: create_new_worker,
            workers,
            received,
        }
    }
}

impl AsyncProcedureCall<UsedHttpClient> for PassingAsyncProcedureCall {
    type Context = UsedContext;
    type Transferables = UsedTransferables;

    fn receive(&self) -> Option<Message<UsedTransferables>> {
        self.received
            .try_borrow_mut()
            .expect("Failed to borrow in receive of APC")
            .pop()
    }

    fn call(&self, input: Input, procedure: AsyncProcedure<Self::Context>) {
        let procedure_ptr = procedure as *mut AsyncProcedure<Self::Context> as u32; // FIXME (wasm-executor): is u32 fine, define an overflow safe function?
        let input = serde_json::to_string(&input).unwrap(); // FIXME (wasm-executor): Remove unwrap

        let array = js_sys::Array::new();
        array.push(&JsValue::from(procedure_ptr));
        array.push(&JsValue::from(input));

        self.workers[0].post_message(&array).unwrap(); // FIXME (wasm-executor): Remove unwrap
    }
}

/// Entry point invoked by the worker.
#[wasm_bindgen]
pub async fn singlethreaded_worker_entry(procedure_ptr: u32, input: String) -> Result<(), JsValue> {
    let procedure: AsyncProcedure<UsedContext> = unsafe { mem::transmute(procedure_ptr) };

    let input = serde_json::from_str::<Input>(&input).unwrap(); // FIXME (wasm-executor): Remove unwrap

    let context = PassingContext {
        source_client: SourceClient::new(HttpSourceClient::new(WHATWGFetchHttpClient::new())),
    };

    (procedure)(input, context).await.unwrap(); // FIXME (wasm-executor): Remove unwrap

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

    let id = in_transfer_obj.get(0).as_f64().unwrap();
    let in_transfer = InTransferMemory {
        type_id: id as u32,
        buffer: in_transfer_obj.get(1).dyn_into().unwrap(),
    };

    let message = Message::<UsedTransferables>::deserialize(
        SerializedMessageTag::from_u32(in_transfer.type_id).unwrap(),
        in_transfer,
    );

    info!("singlethreaded_main_entry {:?}", message.tag());

    // MAJOR FIXME: Fix mutability
    received
        .try_borrow_mut()
        .expect("Failed to borrow in singlethreaded_main_entry")
        .push(message);

    mem::forget(received); // FIXME (wasm-executor): Enforce this somehow

    Ok(())
}
