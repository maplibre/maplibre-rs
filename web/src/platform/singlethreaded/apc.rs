use std::{cell::RefCell, rc::Rc, vec::IntoIter};

use js_sys::{ArrayBuffer, Uint8Array};
use log::error;
use maplibre::{
    environment::OffscreenKernelEnvironment,
    io::{
        apc::{
            AsyncProcedure, AsyncProcedureCall, CallError, Context, Input, IntoMessage, Message,
            SendError,
        },
        source_client::SourceClient,
    },
};
use rand::{prelude::SliceRandom, thread_rng};
use thiserror::Error;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{DedicatedWorkerGlobalScope, Worker};

use crate::{
    error::WebError,
    platform::singlethreaded::{
        transferables::FlatBufferTransferable, UsedContext, UsedHttpClient,
    },
};

/// Error which happens during serialization or deserialization of the tag
#[derive(Error, Debug)]
#[error("failed to deserialize message tag")]
pub struct MessageTagDeserializeError;

#[derive(Clone, Copy, Debug)]
pub enum MessageTag {
    TileTessellated = 1,
    LayerUnavailable = 2,
    LayerTessellated = 3,
    LayerIndexed = 4,
    LayerRaster = 5,
}

impl MessageTag {
    pub fn from_message(message: &Message) -> Self {
        Self::from_u32(message.tag).expect("Unknown message tag encountered") // FIXME tcs: Remove expect
    }

    pub fn from_u32(tag: u32) -> Result<Self, MessageTagDeserializeError> {
        match tag {
            x if x == MessageTag::LayerUnavailable as u32 => Ok(MessageTag::LayerUnavailable),
            x if x == MessageTag::LayerTessellated as u32 => Ok(MessageTag::LayerTessellated),
            x if x == MessageTag::TileTessellated as u32 => Ok(MessageTag::TileTessellated),
            x if x == MessageTag::LayerIndexed as u32 => Ok(MessageTag::LayerIndexed),
            x if x == MessageTag::LayerRaster as u32 => Ok(MessageTag::LayerRaster),
            _ => Err(MessageTagDeserializeError),
        }
    }
}

impl Into<u32> for MessageTag {
    fn into(self) -> u32 {
        self as u32
    }
}

#[derive(Clone)]
pub struct PassingContext {
    pub source_client: SourceClient<UsedHttpClient>,
}

impl Context for PassingContext {
    fn send<T: IntoMessage>(&self, message: T) -> Result<(), SendError> {
        let message = message.into();
        let tag = MessageTag::from_u32(message.tag).unwrap(); // FIXME tcs
        let transferable = message
            .transferable
            .downcast::<FlatBufferTransferable>()
            .expect("Unable to downcast to FlatBufferTransferable"); // FIXME tcs
        let data = transferable.data();

        let buffer = ArrayBuffer::new(data.len() as u32);
        let byte_buffer = Uint8Array::new(&buffer);
        unsafe {
            byte_buffer.set(&Uint8Array::view(data), 0);
        }

        error!("postMessage() during send {:?} {}", tag, data.len());
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
}

pub type ReceivedType = RefCell<Vec<Message>>;

pub struct PassingAsyncProcedureCall {
    workers: Vec<Worker>,

    buffer: RefCell<Vec<Message>>,

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

        Ok(Self {
            workers,
            buffer: RefCell::new(Vec::default()),
            received,
        })
    }
}

impl<K: OffscreenKernelEnvironment> AsyncProcedureCall<K> for PassingAsyncProcedureCall {
    type Context = UsedContext;
    type ReceiveIterator<F: FnMut(&Message) -> bool> = IntoIter<Message>;

    fn receive<F: FnMut(&Message) -> bool>(&self, mut filter: F) -> Self::ReceiveIterator<F> {
        let mut buffer = self.buffer.borrow_mut();
        let mut ret = Vec::new();

        // FIXME: Verify this!
        let mut index = 0usize;
        let mut max_len = buffer.len();
        while index < max_len {
            if filter(&buffer[index]) {
                ret.push(buffer.swap_remove(index));
                max_len = max_len - 1;
            }
            index += 1;
        }

        // TODO: (optimize) Using while instead of if means that we are processing all that is
        // available this might cause frame drops.
        while let Some(message) = self
            .received
            .try_borrow_mut()
            .expect("Failed to borrow in receive of APC")
            .pop()
        {
            // FIXME tracing::debug!("Data reached main thread: {:?}", &message);
            log::warn!("Data reached main thread: {:?}", &message);

            if filter(&message) {
                ret.push(message);
            } else {
                buffer.push(message)
            }
        }

        ret.into_iter()
    }

    fn call(
        &self,
        input: Input,
        procedure: AsyncProcedure<K, UsedContext>,
    ) -> Result<(), CallError> {
        let procedure_ptr = procedure as *mut AsyncProcedure<K, UsedContext> as u32; // FIXME: is u32 fine, define an overflow safe function?
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
