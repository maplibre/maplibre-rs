use crate::environment::DefaultTransferables;
use crate::io::apc::{AsyncProcedure, AsyncProcedureCall, Context, Input, Message};
use crate::io::source_client::{HttpSourceClient, SourceClient};
use crate::io::transferables::Transferables;
use crate::{Environment, HttpClient};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use tokio_util::task::LocalPoolHandle;

// FIXME: Make this generic using the Schedule
#[derive(Clone)]
pub struct TokioContext<T: Transferables, HC: HttpClient> {
    sender: Sender<Message<T>>,
    source_client: SourceClient<HC>,
}

impl<T: Transferables, HC: HttpClient> Context<T, HC> for TokioContext<T, HC>
where
    T: Clone,
{
    fn send(&self, data: Message<T>) {
        self.sender.send(data).unwrap();
    }

    fn source_client(&self) -> &SourceClient<HC> {
        &self.source_client
    }
}

pub struct TokioAsyncProcedureCall<HC: HttpClient> {
    channel: (
        Sender<Message<DefaultTransferables>>,
        Receiver<Message<DefaultTransferables>>,
    ),
    pool: LocalPoolHandle,
    http_client: HC,
}

impl<HC: HttpClient> TokioAsyncProcedureCall<HC> {
    pub fn new(http_client: HC) -> Self {
        Self {
            channel: mpsc::channel(),
            pool: LocalPoolHandle::new(4),
            http_client,
        }
    }
}

impl<HC: HttpClient> AsyncProcedureCall<DefaultTransferables, HC> for TokioAsyncProcedureCall<HC> {
    type Context = TokioContext<DefaultTransferables, HC>;

    fn receive(&mut self) -> Option<Message<DefaultTransferables>> {
        let transferred = self.channel.1.try_recv().ok()?;
        Some(transferred)
    }

    fn schedule(&self, input: Input, procedure: AsyncProcedure<Self::Context>) {
        let sender = self.channel.0.clone();
        let client = self.http_client.clone(); // FIXME: do not clone each time

        self.pool.spawn_pinned(move || async move {
            (procedure)(
                input,
                TokioContext {
                    sender,
                    source_client: SourceClient::Http(HttpSourceClient::new(client)),
                },
            )
            .await;
        });
    }
}
