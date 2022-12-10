use std::{cell::RefCell, rc::Rc};

use js_sys::{ArrayBuffer, Uint8Array};
use log::error;
use maplibre::{
    error::Error,
    io::{
        apc::{AsyncProcedure, AsyncProcedureCall, Context, Input, Message},
        source_client::SourceClient,
    },
};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{DedicatedWorkerGlobalScope, Worker};

use crate::platform::singlethreaded::{
    transferables::FlatBufferTransferable, UsedContext, UsedHttpClient, UsedTransferables,
};

pub struct InTransferMemory {
    pub buffer: Uint8Array,
    pub tag: u32,
}

#[derive(Debug)]
pub enum SerializedMessageTag {
    TileTessellated = 1,
    LayerUnavailable = 2,
    LayerTessellated = 3,
    LayerIndexed = 4,
}

impl SerializedMessageTag {
    pub fn from_u32(tag: u32) -> Option<Self> {
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

#[derive(Clone)]
pub struct PassingContext {
    pub source_client: SourceClient<UsedHttpClient>,
}

impl Context<UsedTransferables, UsedHttpClient> for PassingContext {
    fn send(&self, data: Message<UsedTransferables>) -> Result<(), Error> {
        let tag = match data {
            Message::TileTessellated(_) => SerializedMessageTag::TileTessellated,
            Message::LayerUnavailable(_) => SerializedMessageTag::LayerUnavailable,
            Message::LayerTessellated(_) => SerializedMessageTag::LayerTessellated,
            Message::LayerIndexed(_) => SerializedMessageTag::LayerIndexed,
        };

        let message = match data {
            Message::TileTessellated(message) => {
                let message: FlatBufferTransferable = message;
                message
            }
            Message::LayerUnavailable(message) => {
                let message: FlatBufferTransferable = message;
                message
            }
            Message::LayerTessellated(message) => {
                let message: FlatBufferTransferable = message;
                message
            }
            Message::LayerIndexed(message) => {
                let message: FlatBufferTransferable = message;
                message
            }
        };

        let data = &message.data[message.start..];

        let serialized_array_buffer = ArrayBuffer::new(data.len() as u32);
        let serialized_array = Uint8Array::new(&serialized_array_buffer);
        unsafe {
            serialized_array.set(&Uint8Array::view(data), 0);
        }

        let in_transfer = InTransferMemory {
            buffer: serialized_array,
            tag: tag as u32,
        };

        let global: DedicatedWorkerGlobalScope =
            js_sys::global().dyn_into().map_err(|_e| Error::APC)?;

        // TODO use object
        let transfer_obj = js_sys::Array::new();
        transfer_obj.push(&JsValue::from(in_transfer.tag as u32));
        let buffer = in_transfer.buffer.buffer();
        transfer_obj.push(&buffer);

        let transfer = js_sys::Array::new();
        transfer.push(&buffer);

        // TODO: Verify transfer
        global
            .post_message_with_transfer(&transfer_obj, &transfer)
            .map_err(|e| {
                error!("{:?}", e);
                Error::APC
            })
    }

    fn source_client(&self) -> &SourceClient<UsedHttpClient> {
        &self.source_client
    }
}

pub type ReceivedType = RefCell<Vec<Message<UsedTransferables>>>;

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
