use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{
        mpsc,
        mpsc::{Receiver, Sender},
    },
};

use serde::{Deserialize, Serialize};

use crate::{
    coords::WorldTileCoords,
    io::{
        source_client::{HttpSourceClient, SourceClient},
        transferables::{DefaultTransferables, Transferables},
        TileRequest,
    },
    Environment, HttpClient, Scheduler,
};

/// The result of the tessellation of a tile.
/// `TessellatedLayer` contains the result of the tessellation for a specific layer, otherwise
/// `UnavailableLayer` if the layer doesn't exist.
#[derive(Clone)]
pub enum Message<T: Transferables> {
    TileTessellated(T::TileTessellated),
    UnavailableLayer(T::UnavailableLayer),
    TessellatedLayer(T::TessellatedLayer),
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Input {
    TileRequest(TileRequest),
}

pub trait Context<T: Transferables, HC: HttpClient>: Send + 'static {
    fn send(&self, data: Message<T>);

    fn source_client(&self) -> &SourceClient<HC>;
}

#[cfg(not(feature = "no-thread-safe-futures"))]
pub type AsyncProcedureFuture = Pin<Box<(dyn Future<Output = ()> + Send + 'static)>>;
#[cfg(feature = "no-thread-safe-futures")]
pub type AsyncProcedureFuture = Pin<Box<(dyn Future<Output = ()> + 'static)>>;

pub type AsyncProcedure<C> = fn(input: Input, context: C) -> AsyncProcedureFuture;

pub trait AsyncProcedureCall<T: Transferables, HC: HttpClient>: 'static {
    type Context: Context<T, HC> + Send;

    fn receive(&mut self) -> Option<Message<T>>;

    fn schedule(&self, input: Input, procedure: AsyncProcedure<Self::Context>);
}

#[derive(Clone)]
pub struct SchedulerContext<T: Transferables, HC: HttpClient> {
    sender: Sender<Message<T>>,
    source_client: SourceClient<HC>,
}

impl<T: Transferables, HC: HttpClient> Context<T, HC> for SchedulerContext<T, HC> {
    fn send(&self, data: Message<T>) {
        self.sender.send(data).unwrap(); // FIXME (wasm-executor): Remove unwrap
    }

    fn source_client(&self) -> &SourceClient<HC> {
        &self.source_client
    }
}

pub struct SchedulerAsyncProcedureCall<HC: HttpClient, S: Scheduler> {
    channel: (
        Sender<Message<DefaultTransferables>>,
        Receiver<Message<DefaultTransferables>>,
    ),
    http_client: HC,
    scheduler: S,
}

impl<HC: HttpClient, S: Scheduler> SchedulerAsyncProcedureCall<HC, S> {
    pub fn new(http_client: HC, scheduler: S) -> Self {
        Self {
            channel: mpsc::channel(),
            http_client,
            scheduler,
        }
    }
}

impl<HC: HttpClient, S: Scheduler> AsyncProcedureCall<DefaultTransferables, HC>
    for SchedulerAsyncProcedureCall<HC, S>
{
    type Context = SchedulerContext<DefaultTransferables, HC>;

    fn receive(&mut self) -> Option<Message<DefaultTransferables>> {
        let transferred = self.channel.1.try_recv().ok()?;
        Some(transferred)
    }

    fn schedule(&self, input: Input, procedure: AsyncProcedure<Self::Context>) {
        let sender = self.channel.0.clone();
        let client = self.http_client.clone(); // FIXME (wasm-executor): do not clone each time

        self.scheduler
            .schedule(move || async move {
                (procedure)(
                    input,
                    SchedulerContext {
                        sender,
                        source_client: SourceClient::Http(HttpSourceClient::new(client)),
                    },
                )
                .await;
            })
            .unwrap();
    }
}
