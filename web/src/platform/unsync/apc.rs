use crate::platform::unsync::transferables::{
    InnerData, LinearTessellatedLayer, LinearTransferables,
};
use crate::{MapType, WHATWGFetchHttpClient};
use js_sys::Uint8Array;
use maplibre::environment::Environment;
use maplibre::io::apc::{AsyncProcedure, AsyncProcedureCall, Context, Input, Message};
use maplibre::io::scheduler::Scheduler;
use maplibre::io::source_client::{HttpClient, HttpSourceClient, SourceClient};
use maplibre::io::transferables::Transferables;
use serde::Serialize;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::mem;
use std::mem::{size_of, MaybeUninit};
use std::ops::Deref;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{DedicatedWorkerGlobalScope, Worker};

type UsedTransferables = LinearTransferables;
type UsedHttpClient = WHATWGFetchHttpClient;
type UsedContext = PassingContext;

#[derive(Clone)]
pub struct PassingContext {
    source_client: SourceClient<UsedHttpClient>,
}

impl Context<UsedTransferables, UsedHttpClient> for PassingContext {
    fn send(&self, data: Message<UsedTransferables>) {
        // TODO: send back to main thread via postMessage

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

        let global = js_sys::global().unchecked_into::<DedicatedWorkerGlobalScope>();
        let array = js_sys::Array::new();
        array.push(&JsValue::from(tag));
        array.push(&serialized_array_buffer);
        global.post_message(&array).unwrap();
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
    pub fn new(new_worker: js_sys::Function) -> Self {
        let create_new_worker = Box::new(move || {
            new_worker
                .call0(&JsValue::undefined())
                .unwrap()
                .dyn_into::<Worker>()
                .unwrap()
        });

        let worker = create_new_worker();

        let array = js_sys::Array::new();
        array.push(&wasm_bindgen::module());
        worker.post_message(&array).unwrap();

        Self {
            new_worker: create_new_worker,
            workers: vec![worker],
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
        let procedure_ptr = procedure as *mut AsyncProcedure<Self::Context> as u32; // TODO: is u32 fine?
        let input = serde_json::to_string(&input).unwrap();

        let array = js_sys::Array::new();
        array.push(&JsValue::from(procedure_ptr));
        array.push(&JsValue::from(input));

        self.workers[0].post_message(&array).unwrap();
    }
}

/// Entry point invoked by the worker.
#[wasm_bindgen]
pub async fn unsync_worker_entry(procedure_ptr: u32, input: String) -> Result<(), JsValue> {
    log::info!("worker_entry unsync");

    let procedure: AsyncProcedure<UsedContext> = unsafe { std::mem::transmute(procedure_ptr) };

    let input = serde_json::from_str::<Input>(&input).unwrap();

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
    let mut map = unsafe { Rc::from_raw(map_ptr) };

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
    .unwrap();

    // FIXME: avoid this borrow mess
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

    mem::forget(map);

    Ok(())
}
