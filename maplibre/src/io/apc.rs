use crate::coords::WorldTileCoords;
use crate::io::transferables::Transferables;
use crate::Environment;
use serde::Serialize;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

/// The result of the tessellation of a tile.
/// `TessellatedLayer` contains the result of the tessellation for a specific layer, otherwise
/// `UnavailableLayer` if the layer doesn't exist.
#[derive(Clone)]
pub enum Transferable<T: Transferables> {
    TileTessellated(T::TileTessellated),
    UnavailableLayer(T::UnavailableLayer),
    TessellatedLayer(T::TessellatedLayer),
}

pub trait Context<T: Transferables> {
    fn send(&self, data: Transferable<T>);
}

pub type AsyncProcedure<I, C> =
    fn(input: I, context: C) -> Pin<Box<dyn Future<Output = ()> + Send>>;

pub trait AsyncProcedureCall<T: Transferables> {
    type Context: Context<T> + Send;

    fn new() -> Self;

    fn receive(&self) -> Option<Transferable<T>>;

    fn schedule<I: Send + Serialize + 'static>(
        &self,
        input: I,
        procedure: AsyncProcedure<I, Self::Context>,
    );
}
