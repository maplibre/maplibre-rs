use crate::io::apc::AsyncProcedureCall;
use crate::io::transferables::Transferables;
use crate::io::transferables::{
    DefaultTessellatedLayer, DefaultTileTessellated, DefaultUnavailableLayer,
};
use crate::{HttpClient, MapWindowConfig, Scheduler};

pub trait Environment: 'static {
    type MapWindowConfig: MapWindowConfig;

    type AsyncProcedureCall: AsyncProcedureCall<Self::Transferables>;
    type Scheduler: Scheduler;
    type HttpClient: HttpClient;

    type Transferables: Transferables;
}

#[derive(Copy, Clone)]
pub struct DefaultTransferables;

impl Transferables for DefaultTransferables {
    type TileTessellated = DefaultTileTessellated;
    type UnavailableLayer = DefaultUnavailableLayer;
    type TessellatedLayer = DefaultTessellatedLayer;
}
