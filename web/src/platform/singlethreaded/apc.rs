use std::{cell::RefCell, rc::Rc};

use js_sys::{ArrayBuffer, Uint8Array};
use maplibre::io::{
    apc::{AsyncProcedure, AsyncProcedureCall, CallError, Context, Input, Message, SendError},
    source_client::SourceClient,
};
use rand::{prelude::SliceRandom, thread_rng};
use thiserror::Error;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{DedicatedWorkerGlobalScope, Worker};

use crate::{
    error::WebError,
    platform::singlethreaded::{
        transferables::FlatBufferTransferable, UsedContext, UsedHttpClient, UsedTransferables,
    },
};

/// Error which happens during serialization or deserialization of the tag
#[derive(Error, Debug)]
#[error("failed to deserialize message tag")]
pub struct MessageTagDeserializeError;

#[derive(Debug)]
pub enum MessageTag {
    TileTessellated = 1,
    LayerUnavailable = 2,
    LayerTessellated = 3,
    LayerIndexed = 4,
}

impl MessageTag {
    pub fn from_message(message: &Message<UsedTransferables>) -> Self {
        match message {
            Message::TileTessellated(_) => MessageTag::TileTessellated,
            Message::LayerUnavailable(_) => MessageTag::LayerUnavailable,
            Message::LayerTessellated(_) => MessageTag::LayerTessellated,
            Message::LayerIndexed(_) => MessageTag::LayerIndexed,
        }
    }

    pub fn create_message(
        &self,
        transferable: FlatBufferTransferable,
    ) -> Message<UsedTransferables> {
        // TODO: Verify that data matches tag
        match self {
            MessageTag::TileTessellated => {
                Message::<UsedTransferables>::TileTessellated(transferable)
            }
            MessageTag::LayerUnavailable => {
                Message::<UsedTransferables>::LayerUnavailable(transferable)
            }
            MessageTag::LayerTessellated => {
                Message::<UsedTransferables>::LayerTessellated(transferable)
            }
            MessageTag::LayerIndexed => Message::<UsedTransferables>::LayerIndexed(transferable),
        }
    }

    pub fn from_u32(tag: u32) -> Result<Self, MessageTagDeserializeError> {
        match tag {
            x if x == MessageTag::LayerUnavailable as u32 => Ok(MessageTag::LayerUnavailable),
            x if x == MessageTag::LayerTessellated as u32 => Ok(MessageTag::LayerTessellated),
            x if x == MessageTag::TileTessellated as u32 => Ok(MessageTag::TileTessellated),
            x if x == MessageTag::LayerIndexed as u32 => Ok(MessageTag::LayerIndexed),
            _ => Err(MessageTagDeserializeError),
        }
    }
}

#[derive(Clone)]
pub struct PassingContext {
    pub source_client: SourceClient<UsedHttpClient>,
}

impl Context<UsedTransferables, UsedHttpClient> for PassingContext {
    fn send(&self, message: Message<UsedTransferables>) -> Result<(), SendError> {
        let tag = MessageTag::from_message(&message);
        let transferable = FlatBufferTransferable::from_message(message);
        let data = transferable.data();

        let buffer = ArrayBuffer::new(data.len() as u32);
        let byte_buffer = Uint8Array::new(&buffer);
        unsafe {
            byte_buffer.set(&Uint8Array::view(data), 0);
        }

        let global: DedicatedWorkerGlobalScope = js_sys::global()
            .dyn_into()
            .map_err(|_e| SendError::Transmission)?;
        global
            .post_message_with_transfer(
                &js_sys::Array::of2(&JsValue::from(tag as u32), &buffer),
                &js_sys::Array::of1(&buffer),
            )
            .map_err(|_e| SendError::Transmission)
    }

    fn source_client(&self) -> &SourceClient<UsedHttpClient> {
        &self.source_client
    }
}

pub type ReceivedType = RefCell<Vec<Message<UsedTransferables>>>;

pub struct PassingAsyncProcedureCall {
    workers: Vec<Worker>,

    received: Rc<ReceivedType>, // FIXME: Is RefCell fine?
}

impl PassingAsyncProcedureCall {
    pub fn new(new_worker: js_sys::Function, initial_workers: usize) -> Result<Self, WebError> {
        let received = Rc::new(RefCell::new(vec![]));
        let received_ref = received.clone();

        let create_new_worker = || {
            new_worker
                .call1(
                    &JsValue::undefined(),
                    &JsValue::from(Rc::into_raw(received_ref.clone()) as u32),
                )
                .map_err(WebError::from)?
                .dyn_into::<Worker>()
                .map_err(|_e| WebError::TypeError("Unable to cast to Worker".into()))
        };

        let mut workers = Vec::with_capacity(initial_workers);

        for _ in 0..initial_workers {
            let worker: Worker = create_new_worker()?;

            let array = js_sys::Array::of1(&wasm_bindgen::module());
            worker.post_message(&array).map_err(WebError::from)?;
            workers.push(worker);
        }

        Ok(Self { workers, received })
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

    fn call(
        &self,
        input: Input,
        procedure: AsyncProcedure<Self::Context>,
    ) -> Result<(), CallError> {
        let procedure_ptr = procedure as *mut AsyncProcedure<Self::Context> as u32; // FIXME: is u32 fine, define an overflow safe function?
        let input = serde_json::to_string(&input).map_err(|e| CallError::Serialize(Box::new(e)))?;

        let message = js_sys::Array::of2(&JsValue::from(procedure_ptr), &JsValue::from(input));

        let worker = self
            .workers
            .choose(&mut thread_rng())
            .ok_or(CallError::Schedule)?;

        worker
            .post_message(&message)
            .map_err(|_e| CallError::Schedule)
    }
}
