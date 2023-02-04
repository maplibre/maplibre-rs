use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::{
        mpsc,
        mpsc::{Receiver, Sender},
    },
};

use geozero::mvt::tile;
use image::RgbaImage;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    coords::WorldTileCoords,
    io::{
        geometry_index::{IndexedGeometry, TileIndex},
        pipeline::{PipelineError, PipelineProcessor},
        scheduler::Scheduler,
        source_client::{HttpClient, HttpSourceClient, SourceClient},
        transferables::{
            DefaultTransferables, LayerIndexed, LayerRaster, LayerTessellated, LayerUnavailable,
            TileTessellated, Transferables,
        },
    },
    render::ShaderVertex,
    style::Style,
    tessellation::{IndexDataType, OverAlignedVertexBuffer},
};

/// The result of the tessellation of a tile. This is sent as a message from a worker to the caller
/// of an [`AsyncProcedure`].
///
/// * `TessellatedLayer` contains the result of the tessellation for a specific layer.
/// * `UnavailableLayer` is sent if a requested layer is not found.
/// * `TileTessellated` is sent if processing of a tile finished.
#[derive(Clone)]
pub enum Message<T: Transferables> {
    TileTessellated(T::TileTessellated),
    LayerUnavailable(T::LayerUnavailable),
    LayerTessellated(T::LayerTessellated),
    LayerIndexed(T::LayerIndexed),
    LayerRaster(T::LayerRaster),
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
pub trait Context<T: Transferables, HC: HttpClient>: Send + Clone + 'static {
    /// Send a message back to the caller.
    fn send(&self, data: Message<T>) -> Result<(), SendError>;

    fn source_client(&self) -> &SourceClient<HC>;
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
}

/// Type definitions for asynchronous procedure calls. These functions can be called in an
/// [`AsyncProcedureCall`]. Functions of this type are required to be statically available at
/// compile time. It is explicitly not possible to use closures, as they would require special
/// serialization which is currently not supported.
pub type AsyncProcedure<C> = fn(input: Input, context: C) -> AsyncProcedureFuture;

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
pub trait AsyncProcedureCall<HC: HttpClient>: 'static {
    type Context: Context<Self::Transferables, HC> + Send;
    type Transferables: Transferables;

    /// Try to receive a message non-blocking.
    fn receive(&self) -> Option<Message<Self::Transferables>>;

    /// Call an [`AsyncProcedure`] using some [`Input`]. This function is non-blocking and
    /// returns immediately.
    fn call(&self, input: Input, procedure: AsyncProcedure<Self::Context>)
        -> Result<(), CallError>;
}

#[derive(Clone)]
pub struct SchedulerContext<T: Transferables, HC: HttpClient> {
    sender: Sender<Message<T>>,
    source_client: SourceClient<HC>,
}

impl<T: Transferables, HC: HttpClient> Context<T, HC> for SchedulerContext<T, HC> {
    fn send(&self, data: Message<T>) -> Result<(), SendError> {
        self.sender.send(data).map_err(|_e| SendError::Transmission)
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

impl<HC: HttpClient, S: Scheduler> AsyncProcedureCall<HC> for SchedulerAsyncProcedureCall<HC, S> {
    type Context = SchedulerContext<Self::Transferables, HC>;
    type Transferables = DefaultTransferables;

    fn receive(&self) -> Option<Message<DefaultTransferables>> {
        let transferred = self.channel.1.try_recv().ok()?;
        Some(transferred)
    }

    fn call(
        &self,
        input: Input,
        procedure: AsyncProcedure<Self::Context>,
    ) -> Result<(), CallError> {
        let sender = self.channel.0.clone();
        let client = self.http_client.clone(); // TODO (perf): do not clone each time

        self.scheduler
            .schedule(move || async move {
                procedure(
                    input,
                    SchedulerContext {
                        sender,
                        source_client: SourceClient::new(HttpSourceClient::new(client)),
                    },
                )
                .await
                .unwrap();
            })
            .map_err(|_e| CallError::Schedule)
    }
}

pub struct HeadedPipelineProcessor<T: Transferables, HC: HttpClient, C: Context<T, HC>> {
    context: C,
    phantom_t: PhantomData<T>,
    phantom_hc: PhantomData<HC>,
}

impl<T: Transferables, HC: HttpClient, C: Context<T, HC>> HeadedPipelineProcessor<T, HC, C> {
    pub fn new(context: C) -> Self {
        Self {
            context,
            phantom_t: Default::default(),
            phantom_hc: Default::default(),
        }
    }
}

impl<T: Transferables, HC: HttpClient, C: Context<T, HC>> PipelineProcessor
    for HeadedPipelineProcessor<T, HC, C>
{
    fn tile_finished(&mut self, coords: &WorldTileCoords) -> Result<(), PipelineError> {
        self.context
            .send(Message::TileTessellated(T::TileTessellated::build_from(
                *coords,
            )))
            .map_err(|e| PipelineError::Processing(Box::new(e)))
    }

    fn layer_unavailable(
        &mut self,
        coords: &WorldTileCoords,
        layer_name: &str,
    ) -> Result<(), PipelineError> {
        self.context
            .send(Message::LayerUnavailable(T::LayerUnavailable::build_from(
                *coords,
                layer_name.to_owned(),
            )))
            .map_err(|e| PipelineError::Processing(Box::new(e)))
    }

    fn layer_tesselation_finished(
        &mut self,
        coords: &WorldTileCoords,
        buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
        feature_indices: Vec<u32>,
        layer_data: tile::Layer,
    ) -> Result<(), PipelineError> {
        self.context
            .send(Message::LayerTessellated(T::LayerTessellated::build_from(
                *coords,
                buffer,
                feature_indices,
                layer_data,
            )))
            .map_err(|e| PipelineError::Processing(Box::new(e)))
    }

    fn layer_raster_finished(
        &mut self,
        coords: &WorldTileCoords,
        layer_name: String,
        image_data: RgbaImage,
    ) -> Result<(), PipelineError> {
        self.context
            .send(Message::LayerRaster(T::LayerRaster::build_from(
                *coords, layer_name, image_data,
            )))
            .map_err(|e| PipelineError::Processing(Box::new(e)))
    }

    fn layer_indexing_finished(
        &mut self,
        coords: &WorldTileCoords,
        geometries: Vec<IndexedGeometry<f64>>,
    ) -> Result<(), PipelineError> {
        self.context
            .send(Message::LayerIndexed(T::LayerIndexed::build_from(
                *coords,
                TileIndex::Linear { list: geometries },
            )))
            .map_err(|e| PipelineError::Processing(Box::new(e)))
    }
}
