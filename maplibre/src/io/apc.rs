use std::{
    any::Any,
    cell::RefCell,
    fmt::Debug,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::{
        mpsc,
        mpsc::{Receiver, Sender},
    },
    vec::IntoIter,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    coords::WorldTileCoords, define_label, environment::OffscreenKernelEnvironment,
    io::scheduler::Scheduler, style::Style,
};

define_label!(MessageTag);

impl MessageTag for u32 {
    fn dyn_clone(&self) -> Box<dyn MessageTag> {
        Box::new(*self)
    }
}

#[derive(Error, Debug)]
pub enum MessageError {
    #[error("the message did not contain the expected data")]
    CastError(Box<dyn Any>),
}

/// The result of the tessellation of a tile. This is sent as a message from a worker to the caller
/// of an [`AsyncProcedure`].
#[derive(Debug)]
pub struct Message {
    tag: &'static dyn MessageTag,
    transferable: Box<dyn Any + Send>,
}

impl Message {
    pub fn new(tag: &'static dyn MessageTag, transferable: Box<dyn Any + Send>) -> Self {
        Self { tag, transferable }
    }

    pub fn into_transferable<T: 'static>(self) -> Box<T> {
        self.transferable
            .downcast::<T>()
            .expect("message has wrong tag")
    }

    pub fn has_tag(&self, tag: &'static dyn MessageTag) -> bool {
        self.tag == tag
    }

    pub fn tag(&self) -> &'static dyn MessageTag {
        self.tag
    }
}

pub trait IntoMessage {
    fn into(self) -> Message;
}

/// Inputs for an [`AsyncProcedure`]
#[derive(Clone, Serialize, Deserialize)]
pub enum Input {
    TileRequest {
        coords: WorldTileCoords,
        style: Style, // TODO
    },
    NotYetImplemented, // TODO: Placeholder, should be removed when second input is added
}

#[derive(Error, Debug)]
pub enum SendError {
    #[error("could not transmit data")]
    Transmission,
}

/// Allows sending messages from workers to back to the caller.
pub trait Context: 'static {
    /// Send a message back to the caller.
    fn send<T: IntoMessage>(&self, message: T) -> Result<(), SendError>;
}

#[derive(Error, Debug)]
pub enum ProcedureError {
    /// The [`Input`] is not compatible with the procedure
    #[error("provided input is not compatible with procedure")]
    IncompatibleInput,
    #[error("execution of procedure failed")]
    Execution(Box<dyn std::error::Error>),
    #[error("sending data failed")]
    Send(SendError),
}

#[cfg(feature = "thread-safe-futures")]
pub type AsyncProcedureFuture =
    Pin<Box<(dyn Future<Output = Result<(), ProcedureError>> + Send + 'static)>>;
#[cfg(not(feature = "thread-safe-futures"))]
pub type AsyncProcedureFuture =
    Pin<Box<(dyn Future<Output = Result<(), ProcedureError>> + 'static)>>;

#[derive(Error, Debug)]
pub enum CallError {
    #[error("scheduling work failed")]
    Schedule,
    #[error("serializing data failed")]
    Serialize(Box<dyn std::error::Error>),
    #[error("deserializing failed")]
    Deserialize(Box<dyn std::error::Error>),
    #[error("deserializing input failed")]
    DeserializeInput(Box<dyn std::error::Error>),
}

/// Type definitions for asynchronous procedure calls. These functions can be called in an
/// [`AsyncProcedureCall`]. Functions of this type are required to be statically available at
/// compile time. It is explicitly not possible to use closures, as they would require special
/// serialization which is currently not supported.
pub type AsyncProcedure<K, C> = fn(input: Input, context: C, kernel: K) -> AsyncProcedureFuture;

/// APCs define an interface for performing work asynchronously.
/// This work can be implemented through procedures which can be called asynchronously, hence the
/// name AsyncProcedureCall or APC for short.
///
/// APCs serve as an abstraction for doing work on a separate thread, and then getting responses
/// back. An asynchronous procedure call can for example be performed by using message passing. In
/// fact this could theoretically work over a network socket.
///
/// It is possible to schedule work on a  remote host by calling [`AsyncProcedureCall::call()`]
/// and getting the results back by calling the non-blocking function
/// [`AsyncProcedureCall::receive()`]. The [`AsyncProcedureCall::receive()`] function returns a
/// struct which implements [`Transferables`].
///
/// ## Transferables
///
/// Based on whether the current platform supports shared-memory or not, the implementation of APCs
/// might want to send the whole data from the worker to the caller back or just pointers to that
/// data. The [`Transferables`] trait allows developers to define that and use different data
/// layouts for different platforms.
///
/// ## Message Passing vs APC
///
/// One might wonder why this is called [`AsyncProcedureCall`] instead of `MessagePassingInterface`.
/// The reason for this is quite simple. We are actually referencing and calling procedures which
/// are defined in different threads, processes or hosts. That means, that an [`AsyncProcedureCall`]
/// is actually distinct from a `MessagePassingInterface`.
///
///
/// ## Current Implementations
///
/// We currently have two implementation for APCs. One uses the Tokio async runtime on native
/// targets in [`SchedulerAsyncProcedureCall`].
/// For the web we implemented an alternative way to call APCs which is called
/// [`PassingAsyncProcedureCall`]. This implementation does not depend on shared-memory compared to
/// [`SchedulerAsyncProcedureCall`]. In fact, on the web we are currently not depending on
/// shared-memory because that feature is hidden behind feature flags in browsers
/// (see [here](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer)).
///
///
// TODO: Rename to AsyncProcedureCaller?
pub trait AsyncProcedureCall<K: OffscreenKernelEnvironment>: 'static {
    type Context: Context + Send + Clone;

    type ReceiveIterator<F: FnMut(&Message) -> bool>: Iterator<Item = Message>;

    /// Try to receive a message non-blocking.
    fn receive<F: FnMut(&Message) -> bool>(&self, filter: F) -> Self::ReceiveIterator<F>;

    /// Call an [`AsyncProcedure`] using some [`Input`]. This function is non-blocking and
    /// returns immediately.
    fn call(
        &self,
        input: Input,
        procedure: AsyncProcedure<K, Self::Context>,
    ) -> Result<(), CallError>;
}

#[derive(Clone)]
pub struct SchedulerContext {
    sender: Sender<Message>,
}

impl Context for SchedulerContext {
    fn send<T: IntoMessage>(&self, message: T) -> Result<(), SendError> {
        self.sender
            .send(message.into())
            .map_err(|_e| SendError::Transmission)
    }
}

pub struct SchedulerAsyncProcedureCall<K: OffscreenKernelEnvironment, S: Scheduler> {
    channel: (Sender<Message>, Receiver<Message>),
    buffer: RefCell<Vec<Message>>,
    scheduler: S,
    phantom_k: PhantomData<K>,
}

impl<K: OffscreenKernelEnvironment, S: Scheduler> SchedulerAsyncProcedureCall<K, S> {
    pub fn new(scheduler: S) -> Self {
        Self {
            channel: mpsc::channel(),
            buffer: RefCell::new(Vec::new()),
            phantom_k: PhantomData::default(),
            scheduler,
        }
    }
}

impl<K: OffscreenKernelEnvironment, S: Scheduler> AsyncProcedureCall<K>
    for SchedulerAsyncProcedureCall<K, S>
{
    type Context = SchedulerContext;
    type ReceiveIterator<F: FnMut(&Message) -> bool> = IntoIter<Message>;

    fn receive<F: FnMut(&Message) -> bool>(&self, mut filter: F) -> Self::ReceiveIterator<F> {
        let mut buffer = self.buffer.borrow_mut();
        let mut ret = Vec::new();

        // FIXME tcs: Verify this!
        let mut index = 0usize;
        let mut max_len = buffer.len();
        while index < max_len {
            if filter(&buffer[index]) {
                ret.push(buffer.swap_remove(index));
                max_len -= 1;
            }
            index += 1;
        }

        // TODO: (optimize) Using while instead of if means that we are processing all that is
        // TODO: available this might cause frame drops.
        while let Ok(message) = self.channel.1.try_recv() {
            tracing::debug!("Data reached main thread: {message:?}");
            log::debug!("Data reached main thread: {message:?}");

            if filter(&message) {
                ret.push(message);
            } else {
                buffer.push(message)
            }
        }

        ret.into_iter()
    }

    fn call(
        &self,
        input: Input,
        procedure: AsyncProcedure<K, Self::Context>,
    ) -> Result<(), CallError> {
        let sender = self.channel.0.clone();

        self.scheduler
            .schedule(move || async move {
                log::info!("Processing on thread: {:?}", std::thread::current().name());

                procedure(input, SchedulerContext { sender }, K::create())
                    .await
                    .unwrap();
            })
            .map_err(|_e| CallError::Schedule)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::io::apc::{Context, IntoMessage, SendError};

    pub struct DummyContext;

    impl Context for DummyContext {
        fn send<T: IntoMessage>(&self, _message: T) -> Result<(), SendError> {
            Ok(())
        }
    }
}
