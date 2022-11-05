use std::{cell::RefCell, mem, mem::size_of, rc::Rc, slice};

use js_sys::Uint8Array;
use log::info;
use maplibre::{
    error::Error,
    io::{
        apc::{AsyncProcedure, AsyncProcedureCall, Context, Input, Message},
        source_client::{HttpSourceClient, SourceClient},
        transferables::Transferables,
    },
};
use wasm_bindgen::{prelude::*, JsCast, JsValue};
use web_sys::{DedicatedWorkerGlobalScope, Worker};

use crate::{
    platform::singlethreaded::transferables::{
        InnerData, LinearLayerIndexed, LinearLayerTesselated, LinearLayerUnavailable,
        LinearTileTessellated, LinearTransferables,
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
    fn serialize(&self) -> &[u8];

    fn deserialize(tag: SerializedMessageTag, data: Uint8Array) -> Message<UsedTransferables>;

    fn tag(&self) -> SerializedMessageTag;
}

impl SerializableMessage for Message<LinearTransferables> {
    fn serialize(&self) -> &[u8] {
        unsafe {
            match self {
                // TODO https://github.com/Lokathor/bytemuck/blob/518baf9c0b73c92b4ea4406fe15e005c6d71535a/src/internal.rs#L333
                Message::TileTessellated(message) => slice::from_raw_parts(
                    message as *const LinearTileTessellated as *mut u8,
                    size_of::<LinearTileTessellated>(),
                ),
                Message::LayerUnavailable(message) => slice::from_raw_parts(
                    message as *const LinearLayerUnavailable as *mut u8,
                    size_of::<LinearLayerUnavailable>(),
                ),
                Message::LayerTessellated(message) => slice::from_raw_parts(
                    message.data.as_ref() as *const InnerData as *mut u8,
                    size_of::<InnerData>(),
                ),
                Message::LayerIndexed(message) => slice::from_raw_parts(
                    message as *const LinearLayerIndexed as *mut u8,
                    size_of::<LinearLayerIndexed>(),
                ),
            }
        }
    }

    fn deserialize(tag: SerializedMessageTag, data: Uint8Array) -> Message<UsedTransferables> {
        type TileTessellated = <UsedTransferables as Transferables>::TileTessellated;
        type UnavailableLayer = <UsedTransferables as Transferables>::LayerUnavailable;
        type IndexedLayer = <UsedTransferables as Transferables>::LayerIndexed;
        unsafe {
            // TODO: https://github.com/Lokathor/bytemuck/blob/518baf9c0b73c92b4ea4406fe15e005c6d71535a/src/internal.rs#L159
            match tag {
                SerializedMessageTag::TileTessellated => {
                    Message::<UsedTransferables>::TileTessellated(
                        (&*(data.to_vec().as_slice() as *const [u8] as *const TileTessellated))
                            .clone(),
                    )
                }
                SerializedMessageTag::LayerUnavailable => {
                    Message::<UsedTransferables>::LayerUnavailable(
                        (&*(data.to_vec().as_slice() as *const [u8] as *const UnavailableLayer))
                            .clone(),
                    )
                }
                SerializedMessageTag::LayerTessellated => {
                    Message::<UsedTransferables>::LayerTessellated(LinearLayerTesselated {
                        data: unsafe {
                            let mut uninit = Box::<InnerData>::new_zeroed();
                            data.raw_copy_to_ptr(uninit.as_mut_ptr() as *mut u8);

                            uninit.assume_init()
                        },
                    })
                }
                SerializedMessageTag::LayerIndexed => Message::<UsedTransferables>::LayerIndexed(
                    (&*(data.to_vec().as_slice() as *const [u8] as *const IndexedLayer)).clone(),
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
        let tag = data.tag();
        let serialized = data.serialize();

        let serialized_array_buffer = js_sys::ArrayBuffer::new(serialized.len() as u32);
        let serialized_array = js_sys::Uint8Array::new(&serialized_array_buffer);
        unsafe {
            serialized_array.set(&Uint8Array::view(serialized), 0);
        }

        let global: DedicatedWorkerGlobalScope =
            js_sys::global().dyn_into().map_err(|_e| Error::APC)?;
        let array = js_sys::Array::new();
        array.push(&JsValue::from(tag as u32));
        array.push(&serialized_array_buffer);
        global.post_message(&array).map_err(|_e| Error::APC)
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
        self.received.borrow_mut().pop()
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
    let procedure: AsyncProcedure<UsedContext> = unsafe { std::mem::transmute(procedure_ptr) };

    let input = serde_json::from_str::<Input>(&input).unwrap(); // FIXME (wasm-executor): Remove unwrap

    let context = PassingContext {
        source_client: SourceClient::new(HttpSourceClient::new(WHATWGFetchHttpClient::new())),
    };

    (procedure)(input, context).await;

    Ok(())
}

/// Entry point invoked by the main thread.
#[wasm_bindgen]
pub unsafe fn singlethreaded_main_entry(
    received_ptr: *const ReceivedType,
    type_id: u32,
    data: Uint8Array,
) -> Result<(), JsValue> {
    // FIXME (wasm-executor): Can we make this call safe? check if it was cloned before?
    let received: Rc<ReceivedType> = Rc::from_raw(received_ptr);

    let message = Message::<UsedTransferables>::deserialize(
        SerializedMessageTag::from_u32(type_id).unwrap(),
        data,
    );

    info!("singlethreaded_main_entry {:?}", message.tag());

    // MAJOR FIXME: Fix mutability
    received.borrow_mut().push(message);

    mem::forget(received); // FIXME (wasm-executor): Enforce this somehow

    Ok(())
}
