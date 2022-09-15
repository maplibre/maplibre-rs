use crate::platform::sync::pool_scheduler::WebWorkerPoolScheduler;
use maplibre::environment::DefaultTransferables;
use maplibre::environment::Environment;
use maplibre::io::apc::{AsyncProcedure, AsyncProcedureCall, Context, Message};
use maplibre::io::scheduler::Scheduler;
use maplibre::io::source_client::{HttpClient, HttpSourceClient, SourceClient};
use maplibre::io::transferables::Transferables;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

#[derive(Clone)]
pub struct AtomicContext<T: Transferables, HC: HttpClient> {
    sender: Sender<Message<T>>,
    source_client: SourceClient<HC>,
}

impl<T: Transferables, HC: HttpClient> Context<T, HC> for AtomicContext<T, HC>
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

pub struct AtomicAsyncProcedureCall {
    channel: (
        Sender<Message<DefaultTransferables>>,
        Receiver<Message<DefaultTransferables>>,
    ),
    scheduler: WebWorkerPoolScheduler,
}

impl AtomicAsyncProcedureCall {
    pub fn new(scheduler: WebWorkerPoolScheduler) -> Self {
        Self {
            channel: mpsc::channel(),
            scheduler,
        }
    }
}

impl<HC: HttpClient> AsyncProcedureCall<DefaultTransferables, HC> for AtomicAsyncProcedureCall {
    type Context = AtomicContext<DefaultTransferables, HC>;

    fn receive(&self) -> Option<Message<DefaultTransferables>> {
        let transferred = self.channel.1.try_recv().ok()?;
        Some(transferred)
    }

    fn schedule<I: Serialize + Send + 'static>(
        &self,
        input: I,
        procedure: AsyncProcedure<I, AtomicContext<DefaultTransferables, HC>>,
        http_client: HttpSourceClient<HC>,
    ) {
        let sender = self.channel.0.clone();

        self.scheduler
            .schedule(move || async move {
                (procedure)(
                    input,
                    AtomicContext {
                        sender,
                        source_client: SourceClient::Http(http_client),
                    },
                )
                .await;
            })
            .unwrap();
    }
}
