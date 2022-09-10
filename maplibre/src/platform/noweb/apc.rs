use crate::environment::DefaultTransferables;
use crate::io::apc::{AsyncProcedure, AsyncProcedureCall, Context, Transferable};
use crate::io::transferables::Transferables;
use crate::Environment;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

#[derive(Clone)]
pub struct TokioContext<T: Transferables> {
    sender: Sender<Transferable<T>>,
}

impl<T: Transferables> Context<T> for TokioContext<T>
where
    T: Clone,
{
    fn send(&self, data: Transferable<T>) {
        self.sender.send(data).unwrap();
        log::debug!("sent");
    }
}

pub struct TokioAsyncProcedureCall<T: Transferables> {
    channel: (Sender<Transferable<T>>, Receiver<Transferable<T>>),
}

impl<T: Transferables> TokioAsyncProcedureCall<T> {
    pub fn new() -> Self {
        Self {
            channel: mpsc::channel(),
        }
    }
}

impl<T: Transferables> AsyncProcedureCall<T> for TokioAsyncProcedureCall<T>
where
    T: Clone,
{
    type Context = TokioContext<T>;

    fn new() -> Self {
        Self {
            channel: mpsc::channel(),
        }
    }

    fn receive(&self) -> Option<Transferable<T>> {
        let transferred = self.channel.1.try_recv().ok()?;
        log::debug!("received");
        Some(transferred)
    }

    fn schedule<I: Serialize + Send + 'static>(
        &self,
        input: I,
        procedure: AsyncProcedure<I, TokioContext<T>>,
    ) {
        let sender = self.channel.0.clone();
        tokio::task::spawn(async move {
            (procedure)(input, TokioContext { sender }).await;
        });
    }
}
