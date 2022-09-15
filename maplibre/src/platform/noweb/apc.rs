use crate::environment::DefaultTransferables;
use crate::io::apc::{AsyncProcedure, AsyncProcedureCall, Context, Input, Transferable};
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
    sender: Sender<Transferable<T>>,
    source_client: SourceClient<HC>,
}

impl<T: Transferables, HC: HttpClient> Context<T, HC> for TokioContext<T, HC>
where
    T: Clone,
{
    fn send(&self, data: Transferable<T>) {
        self.sender.send(data).unwrap();
        log::debug!("sent");
    }

    fn source_client(&self) -> &SourceClient<HC> {
        &self.source_client
    }
}

pub struct TokioAsyncProcedureCall {
    channel: (
        Sender<Transferable<DefaultTransferables>>,
        Receiver<Transferable<DefaultTransferables>>,
    ),
    pool: LocalPoolHandle,
}

impl TokioAsyncProcedureCall {
    pub fn new() -> Self {
        Self {
            channel: mpsc::channel(),
            pool: LocalPoolHandle::new(4),
        }
    }
}

impl<HC: HttpClient> AsyncProcedureCall<DefaultTransferables, HC> for TokioAsyncProcedureCall {
    type Context = TokioContext<DefaultTransferables, HC>;

    fn receive(&mut self) -> Option<Box<Transferable<DefaultTransferables>>> {
        let transferred = self.channel.1.try_recv().ok()?;
        log::debug!("received");
        Some(Box::new(transferred))
    }

    fn schedule(
        &self,
        input: Input,
        procedure: AsyncProcedure<DefaultTransferables, HC>,
        http_client: HttpSourceClient<HC>,
    ) {
        let sender = self.channel.0.clone();

        self.pool.spawn_pinned(move || async move {
            (procedure)(
                input,
                Box::new(TokioContext {
                    sender,
                    source_client: SourceClient::Http(http_client),
                }),
            )
            .await;
        });
    }
}
