use crate::coords::WorldTileCoords;
use crate::io::source_client::{HttpSourceClient, SourceClient};
use crate::io::transferables::Transferables;
use crate::io::TileRequest;
use crate::{Environment, HttpClient};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

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
