use std::{
    any::TypeId,
    borrow::Borrow,
    cell::RefCell,
    collections::HashMap,
    marker::PhantomData,
    mem,
    mem::{size_of, MaybeUninit},
    ops::Deref,
    pin::Pin,
    rc::Rc,
    sync::{
        mpsc,
        mpsc::{Receiver, Sender},
    },
};

use js_sys::Uint8Array;
use maplibre::{
    environment::Environment,
    io::{
        apc::{AsyncProcedure, AsyncProcedureCall, Context, Input, Message},
        scheduler::Scheduler,
        source_client::{HttpClient, HttpSourceClient, SourceClient},
        transferables::Transferables,
    },
};
use wasm_bindgen::{prelude::*, JsCast, JsValue};
use web_sys::{DedicatedWorkerGlobalScope, Worker};

use crate::{
    platform::singlethreaded::transferables::{
        InnerData, LinearTessellatedLayer, LinearTransferables,
    },
    MapType, WHATWGFetchHttpClient,
};

type UsedTransferables = LinearTransferables;
type UsedHttpClient = WHATWGFetchHttpClient;
type UsedContext = PassingContext;

enum SerializedMessageTag {
    TileTessellated = 1,
    UnavailableLayer = 2,
    TessellatedLayer = 3,
}

impl SerializedMessageTag {
    fn from_u32(tag: u32) -> Option<Self> {
        match tag {
            x if x == SerializedMessageTag::UnavailableLayer as u32 => {
                Some(SerializedMessageTag::UnavailableLayer)
            }
            x if x == SerializedMessageTag::TessellatedLayer as u32 => {
                Some(SerializedMessageTag::TessellatedLayer)
            }
            x if x == SerializedMessageTag::TileTessellated as u32 => {
                Some(SerializedMessageTag::TileTessellated)
            }
            _ => None,
        }
    }
}

trait SerializableMessage {
    fn serialize(&self) -> &[u8];

    fn deserialize(tag: SerializedMessageTag, data: Uint8Array) -> Message<UsedTransferables>;

    fn tag(&self) -> SerializedMessageTag;
}

impl SerializableMessage for Message<LinearTransferables> {
    fn serialize(&self) -> &[u8] {
        match self {
            Message::TileTessellated(data) => bytemuck::bytes_of(data),
            Message::UnavailableLayer(data) => bytemuck::bytes_of(data),
            Message::TessellatedLayer(data) => bytemuck::bytes_of(data.data.as_ref()),
        }
    }

    fn deserialize(tag: SerializedMessageTag, data: Uint8Array) -> Message<UsedTransferables> {
        match tag {
            SerializedMessageTag::TileTessellated => {
                Message::<UsedTransferables>::TileTessellated(*bytemuck::from_bytes::<
                    <UsedTransferables as Transferables>::TileTessellated,
                >(&data.to_vec()))
            }
            SerializedMessageTag::UnavailableLayer => {
                Message::<UsedTransferables>::UnavailableLayer(*bytemuck::from_bytes::<
                    <UsedTransferables as Transferables>::UnavailableLayer,
                >(&data.to_vec()))
            }
            SerializedMessageTag::TessellatedLayer => {
                Message::<UsedTransferables>::TessellatedLayer(LinearTessellatedLayer {
                    data: unsafe {
                        let mut uninit = Box::<InnerData>::new_zeroed();
                        data.raw_copy_to_ptr(uninit.as_mut_ptr() as *mut u8);
                        let x = uninit.assume_init();

                        x
                    },
                })
            }
        }
    }

    fn tag(&self) -> SerializedMessageTag {
        match self {
            Message::TileTessellated(_) => SerializedMessageTag::TileTessellated,
            Message::UnavailableLayer(_) => SerializedMessageTag::UnavailableLayer,
            Message::TessellatedLayer(_) => SerializedMessageTag::TessellatedLayer,
        }
    }
}

#[derive(Clone)]
pub struct PassingContext {
    source_client: SourceClient<UsedHttpClient>,
}

impl Context<UsedTransferables, UsedHttpClient> for PassingContext {
    fn send(&self, data: Message<UsedTransferables>) {
        let tag = data.tag();
        let serialized = data.serialize();

        let serialized_array_buffer = js_sys::ArrayBuffer::new(serialized.len() as u32);
        let serialized_array = js_sys::Uint8Array::new(&serialized_array_buffer);
        unsafe {
            serialized_array.set(&Uint8Array::view(serialized), 0);
        }

        let global = js_sys::global().unchecked_into::<DedicatedWorkerGlobalScope>(); // FIXME (wasm-executor): Remove unchecked
        let array = js_sys::Array::new();
        array.push(&JsValue::from(tag as u32));
        array.push(&serialized_array_buffer);
        global.post_message(&array).unwrap(); // FIXME (wasm-executor) Remove unwrap
    }

    fn source_client(&self) -> &SourceClient<UsedHttpClient> {
        &self.source_client
    }
}

pub struct PassingAsyncProcedureCall {
    new_worker: Box<dyn Fn() -> Worker>,
    workers: Vec<Worker>,

    received: Vec<Message<UsedTransferables>>,
}

impl PassingAsyncProcedureCall {
    pub fn new(new_worker: js_sys::Function, initial_workers: u8) -> Self {
        let create_new_worker = Box::new(move || {
            new_worker
                .call0(&JsValue::undefined())
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
            received: vec![],
        }
    }
}

impl AsyncProcedureCall<UsedTransferables, UsedHttpClient> for PassingAsyncProcedureCall {
    type Context = UsedContext;

    fn receive(&mut self) -> Option<Message<UsedTransferables>> {
        self.received.pop()
    }

    fn schedule(&self, input: Input, procedure: AsyncProcedure<Self::Context>) {
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
    let procedure: AsyncProcedure<UsedContext> = unsafe { std::mem::transmute(procedure_ptr) };

    let input = serde_json::from_str::<Input>(&input).unwrap(); // FIXME (wasm-executor): Remove unwrap

    let context = PassingContext {
        source_client: SourceClient::Http(HttpSourceClient::new(WHATWGFetchHttpClient::new())),
    };

    (procedure)(input, context).await;

    Ok(())
}

/// Entry point invoked by the main thread.
#[wasm_bindgen]
pub unsafe fn singlethreaded_main_entry(
    map_ptr: *const RefCell<MapType>,
    type_id: u32,
    data: Uint8Array,
) -> Result<(), JsValue> {
    // FIXME (wasm-executor): Can we make this call safe? check if it was cloned before?
    let mut map = Rc::from_raw(map_ptr);

    let message = Message::<UsedTransferables>::deserialize(
        SerializedMessageTag::from_u32(type_id).unwrap(),
        data,
    );

    map.deref()
        .borrow()
        .map_schedule()
        .deref()
        .borrow()
        .apc
        .deref()
        .borrow_mut()
        .received
        .push(message);

    mem::forget(map); // FIXME (wasm-executor): Enforce this somehow

    Ok(())
}
