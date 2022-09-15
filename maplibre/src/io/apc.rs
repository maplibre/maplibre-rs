use crate::coords::WorldTileCoords;
use crate::environment::DefaultTransferables;
use crate::io::source_client::{HttpSourceClient, SourceClient};
use crate::io::transferables::Transferables;
use crate::io::TileRequest;
use crate::Scheduler;
use crate::{Environment, HttpClient};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

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

pub trait Context<T: Transferables, HC: HttpClient>: 'static {
    fn send(&self, data: Message<T>);

    fn source_client(&self) -> &SourceClient<HC>;
}

pub type AsyncProcedure<C> = fn(input: Input, context: C) -> Pin<Box<dyn Future<Output = ()>>>;

pub trait AsyncProcedureCall<T: Transferables, HC: HttpClient>: 'static {
    type Context: Context<T, HC> + Send;

    fn receive(&mut self) -> Option<Message<T>>;

    fn schedule(&self, input: Input, procedure: AsyncProcedure<Self::Context>);
}

// FIXME: Make this generic using the Schedule
#[derive(Clone)]
pub struct TokioContext<T: Transferables, HC: HttpClient> {
    sender: Sender<Message<T>>,
    source_client: SourceClient<HC>,
}

impl<T: Transferables, HC: HttpClient> Context<T, HC> for TokioContext<T, HC> {
    fn send(&self, data: Message<T>) {
        self.sender.send(data).unwrap();
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
    type Context = TokioContext<DefaultTransferables, HC>;

    fn receive(&mut self) -> Option<Message<DefaultTransferables>> {
        let transferred = self.channel.1.try_recv().ok()?;
        Some(transferred)
    }

    fn schedule(&self, input: Input, procedure: AsyncProcedure<Self::Context>) {
        let sender = self.channel.0.clone();
        let client = self.http_client.clone(); // FIXME: do not clone each time

        self.scheduler
            .schedule(move || async move {
                (procedure)(
                    input,
                    TokioContext {
                        sender,
                        source_client: SourceClient::Http(HttpSourceClient::new(client)),
                    },
                )
                .await;
            })
            .unwrap();
    }
}
