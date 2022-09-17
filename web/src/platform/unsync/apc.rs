use std::{
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
    platform::unsync::transferables::{InnerData, LinearTessellatedLayer, LinearTransferables},
    MapType, WHATWGFetchHttpClient,
};

type UsedTransferables = LinearTransferables;
type UsedHttpClient = WHATWGFetchHttpClient;
type UsedContext = PassingContext;

#[derive(Clone)]
pub struct PassingContext {
    source_client: SourceClient<UsedHttpClient>,
}

impl Context<UsedTransferables, UsedHttpClient> for PassingContext {
    fn send(&self, data: Message<UsedTransferables>) {
        let (tag, serialized): (u32, &[u8]) = match &data {
            Message::TileTessellated(data) => (1, bytemuck::bytes_of(data)),
            Message::UnavailableLayer(data) => (2, bytemuck::bytes_of(data)),
            Message::TessellatedLayer(data) => (3, bytemuck::bytes_of(data.data.as_ref())),
        };

        let serialized_array_buffer = js_sys::ArrayBuffer::new(serialized.len() as u32);
        let serialized_array = js_sys::Uint8Array::new(&serialized_array_buffer);
        unsafe {
            serialized_array.set(&Uint8Array::view(serialized), 0);
        }

        let global = js_sys::global().unchecked_into::<DedicatedWorkerGlobalScope>(); // FIXME (wasm-executor): Remove unchecked
        let array = js_sys::Array::new();
        array.push(&JsValue::from(tag));
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
pub async fn unsync_worker_entry(procedure_ptr: u32, input: String) -> Result<(), JsValue> {
    log::info!("worker_entry unsync");

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
pub fn unsync_main_entry(
    map_ptr: *const RefCell<MapType>,
    tag: u32,
    data: Uint8Array,
) -> Result<(), JsValue> {
    // FIXME (wasm-executor): Can we make this call safe? check if it was cloned before?
    let mut map = unsafe { Rc::from_raw(map_ptr) };

    // FIXME (wasm-executor): remove tag somehow
    let transferred = match tag {
        3 => Some(Message::<UsedTransferables>::TessellatedLayer(
            LinearTessellatedLayer {
                data: unsafe {
                    let mut uninit = Box::<InnerData>::new_zeroed();
                    data.raw_copy_to_ptr(uninit.as_mut_ptr() as *mut u8);
                    let x = uninit.assume_init();

                    x
                },
            },
        )),
        1 => Some(Message::<UsedTransferables>::TileTessellated(
            *bytemuck::from_bytes::<<UsedTransferables as Transferables>::TileTessellated>(
                &data.to_vec(),
            ),
        )),
        2 => Some(Message::<UsedTransferables>::UnavailableLayer(
            *bytemuck::from_bytes::<<UsedTransferables as Transferables>::UnavailableLayer>(
                &data.to_vec(),
            ),
        )),
        _ => None,
    }
    .unwrap(); // FIXME (wasm-executor): Remove unwrap

    map.deref()
        .borrow()
        .map_schedule()
        .deref()
        .borrow()
        .apc
        .deref()
        .borrow_mut()
        .received
        .push(transferred);

    mem::forget(map); // FIXME (wasm-executor): Enforce this somehow

    Ok(())
}
