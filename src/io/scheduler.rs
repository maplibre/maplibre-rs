use std::collections::HashSet;
use std::future::Future;

use geozero::mvt::Tile;
use geozero::GeozeroDatasource;
use std::sync::mpsc::{channel, Receiver, SendError, Sender};
use std::sync::{Arc, Mutex};

use vector_tile::parse_tile_bytes;

use crate::coords::{WorldCoords, WorldTileCoords, Zoom};
use crate::io::tile_cache::TileCache;
use crate::io::{
    LayerTessellateMessage, TessellateMessage, TileFetchResult, TileRequest, TileRequestID,
    TileTessellateMessage,
};

use crate::error::Error;
use crate::io::geometry_index::{GeometryIndex, IndexProcessor, IndexedGeometry, TileIndex};
use crate::io::shared_thread_state::SharedThreadState;
use crate::io::source_client::{HttpSourceClient, SourceClient};
use crate::io::tile_request_state::TileRequestState;
use crate::tessellation::Tessellated;
use prost::Message;

pub struct Scheduler {
    schedule_method: ScheduleMethod,
}

impl Scheduler {
    pub fn new(schedule_method: ScheduleMethod) -> Self {
        Self { schedule_method }
    }

    pub fn schedule_method(&self) -> &ScheduleMethod {
        &self.schedule_method
    }
}

pub enum ScheduleMethod {
    #[cfg(not(target_arch = "wasm32"))]
    Tokio(crate::platform::schedule_method::TokioScheduleMethod),
    #[cfg(target_arch = "wasm32")]
    WebWorkerPool(crate::platform::schedule_method::WebWorkerPoolScheduleMethod),
}

impl Default for ScheduleMethod {
    fn default() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            ScheduleMethod::Tokio(crate::platform::schedule_method::TokioScheduleMethod::new())
        }
        #[cfg(target_arch = "wasm32")]
        {
            panic!("No default ScheduleMethod on web")
        }
    }
}

impl ScheduleMethod {
    #[cfg(target_arch = "wasm32")]
    pub fn schedule<T>(
        &self,
        shared_thread_state: SharedThreadState,
        future_factory: impl (FnOnce(SharedThreadState) -> T) + Send + 'static,
    ) -> Result<(), Error>
    where
        T: Future<Output = ()> + 'static,
    {
        match self {
            ScheduleMethod::WebWorkerPool(method) => {
                Ok(method.schedule(shared_thread_state, future_factory))
            }
            _ => Err(Error::Schedule),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn schedule<T>(
        &self,
        shared_thread_state: SharedThreadState,
        future_factory: impl (FnOnce(SharedThreadState) -> T) + Send + 'static,
    ) -> Result<(), Error>
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static,
    {
        match self {
            ScheduleMethod::Tokio(method) => {
                method.schedule(shared_thread_state, future_factory);
                Ok(())
            }
            _ => Err(Error::Schedule),
        }
    }
}
