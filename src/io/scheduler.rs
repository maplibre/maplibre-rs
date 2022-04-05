use std::collections::{HashMap, HashSet};
use std::future::Future;

use geozero::mvt::Tile;
use geozero::GeozeroDatasource;
use log::{error, info};
use std::sync::mpsc::{channel, Receiver, SendError, Sender};
use std::sync::{Arc, Mutex};

use style_spec::source::TileAddressingScheme;
use vector_tile::parse_tile_bytes;

/// Describes through which channels work-requests travel. It describes the flow of work.
use crate::coords::{TileCoords, WorldTileCoords};
use crate::io::tile_cache::TileCache;
use crate::io::{
    LayerTessellateMessage, TessellateMessage, TileFetchResult, TileRequest, TileRequestID,
    TileTessellateMessage,
};

use crate::error::Error;
use crate::io::geometry_index::{GeometryIndex, IndexProcessor, TileIndex};
use crate::io::source_client::{HttpSourceClient, SourceClient};
use crate::tessellation::Tessellated;
use prost::Message;

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
        scheduler: &IOScheduler,
        future_factory: impl (FnOnce(ThreadLocalTessellatorState) -> T) + Send + 'static,
    ) -> Result<(), Error>
    where
        T: Future<Output = ()> + 'static,
    {
        match self {
            ScheduleMethod::WebWorkerPool(method) => Ok(method.schedule(scheduler, future_factory)),
            _ => Err(Error::Schedule),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn schedule<T>(
        &self,
        scheduler: &IOScheduler,
        future_factory: impl (FnOnce(ThreadLocalTessellatorState) -> T) + Send + 'static,
    ) -> Result<(), Error>
    where
        T: std::future::Future + Send + 'static,
        T::Output: Send + 'static,
    {
        match self {
            ScheduleMethod::Tokio(method) => Ok(method.schedule(scheduler, future_factory)),
            _ => Err(Error::Schedule),
        }
    }
}

pub struct ThreadLocalTessellatorState {
    tile_request_state: Arc<Mutex<TileRequestState>>,
    tessellate_result_sender: Sender<TessellateMessage>,
    geometry_index: Arc<Mutex<GeometryIndex>>,
}

#[cfg(target_arch = "wasm32")]
impl Drop for ThreadLocalTessellatorState {
    fn drop(&mut self) {
        use log::warn;
        warn!(
            "ThreadLocalTessellatorState dropped. \
            On web this should only happen when the application is stopped!"
        );
    }
}

impl ThreadLocalTessellatorState {
    fn get_tile_request(&self, request_id: TileRequestID) -> Option<TileRequest> {
        self.tile_request_state
            .lock()
            .ok()
            .and_then(|tile_request_state| tile_request_state.get_tile_request(request_id).cloned())
    }

    #[tracing::instrument(skip(self, data))]
    pub fn process_tile(
        &self,
        request_id: TileRequestID,
        data: Box<[u8]>,
    ) -> Result<(), SendError<TessellateMessage>> {
        if let Some(tile_request) = self.get_tile_request(request_id) {
            let tile_result = TileFetchResult::Tile {
                coords: tile_request.coords,
                data,
            };

            self.tessellate_layers_with_request(&tile_result, &tile_request, request_id)?;
            self.index_geometry(&tile_result);
        }

        Ok(())
    }

    pub fn tile_unavailable(
        &self,
        request_id: TileRequestID,
    ) -> Result<(), SendError<TessellateMessage>> {
        if let Some(tile_request) = self.get_tile_request(request_id) {
            let tile_result = TileFetchResult::Unavailable {
                coords: tile_request.coords,
            };
            self.tessellate_layers_with_request(&tile_result, &tile_request, request_id)?;
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn index_geometry(&self, tile_result: &TileFetchResult) {
        match tile_result {
            TileFetchResult::Tile { data, coords } => {
                let tile: Tile = Tile::decode(data.as_ref()).unwrap();

                let mut processor = IndexProcessor::new();
                for mut layer in tile.layers {
                    layer.process(&mut processor).unwrap();
                }

                if let Some(mut geometry_index) = self.geometry_index.lock().ok() {
                    geometry_index.index_tile(
                        &coords,
                        TileIndex::Linear {
                            list: processor.get_geometries(),
                        },
                    )
                }
            }
            _ => {}
        }
    }

    #[tracing::instrument(skip(self))]
    fn tessellate_layers_with_request(
        &self,
        tile_result: &TileFetchResult,
        tile_request: &TileRequest,
        request_id: TileRequestID,
    ) -> Result<(), SendError<TessellateMessage>> {
        match tile_result {
            TileFetchResult::Unavailable { coords } => {
                for to_load in &tile_request.layers {
                    self.tessellate_result_sender
                        .send(TessellateMessage::Layer(
                            LayerTessellateMessage::UnavailableLayer {
                                coords: *coords,
                                layer_name: to_load.to_string(),
                            },
                        ))?;
                }
            }
            TileFetchResult::Tile { data, coords } => {
                info!("parsing tile {} with {}bytes", &coords, data.len());
                let tile = parse_tile_bytes(data).expect("failed to load tile");

                for to_load in &tile_request.layers {
                    if let Some(layer) = tile
                        .layers()
                        .iter()
                        .find(|layer| to_load.as_str() == layer.name())
                    {
                        match layer.tessellate() {
                            Ok((buffer, feature_indices)) => {
                                self.tessellate_result_sender
                                    .send(TessellateMessage::Layer(
                                        LayerTessellateMessage::TessellatedLayer {
                                            coords: *coords,
                                            buffer: buffer.into(),
                                            feature_indices,
                                            layer_data: layer.clone(),
                                        },
                                    ))?;
                            }
                            Err(e) => {
                                self.tessellate_result_sender
                                    .send(TessellateMessage::Layer(
                                        LayerTessellateMessage::UnavailableLayer {
                                            coords: *coords,
                                            layer_name: to_load.to_string(),
                                        },
                                    ))?;

                                error!(
                                    "tesselation for layer {} failed: {} {:?}",
                                    to_load, &coords, e
                                );
                            }
                        }

                        info!("layer {} ready: {}", to_load, &coords);
                    } else {
                        self.tessellate_result_sender
                            .send(TessellateMessage::Layer(
                                LayerTessellateMessage::UnavailableLayer {
                                    coords: *coords,
                                    layer_name: to_load.to_string(),
                                },
                            ))?;

                        info!("layer {} not found: {}", to_load, &coords);
                    }
                }
            }
        }

        self.tessellate_result_sender
            .send(TessellateMessage::Tile(TileTessellateMessage {
                request_id,
            }))?;

        Ok(())
    }
}

pub struct IOScheduler {
    tessellate_channel: (Sender<TessellateMessage>, Receiver<TessellateMessage>),
    tile_request_state: Arc<Mutex<TileRequestState>>,
    geometry_index: Arc<Mutex<GeometryIndex>>,
    tile_cache: TileCache,
    schedule_method: ScheduleMethod,
}

const _: () = {
    fn assert_send<T: Send>() {}

    fn assert_all() {
        assert_send::<ThreadLocalTessellatorState>();
    }
};

impl IOScheduler {
    pub fn new(schedule_method: ScheduleMethod) -> Self {
        Self {
            tessellate_channel: channel(),
            tile_request_state: Arc::new(Mutex::new(TileRequestState::new())),
            geometry_index: Arc::new(Mutex::new(GeometryIndex::new())),
            tile_cache: TileCache::new(),
            schedule_method,
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn try_populate_cache(&mut self) {
        if let Ok(mut tile_request_state) = self.tile_request_state.try_lock() {
            if let Ok(result) = self.tessellate_channel.1.try_recv() {
                match result {
                    TessellateMessage::Tile(TileTessellateMessage { request_id }) => {
                        tile_request_state.finish_tile_request(request_id);
                    }
                    TessellateMessage::Layer(layer_result) => {
                        self.tile_cache.put_tesselated_layer(layer_result);
                    }
                }
            }
        }
    }

    pub fn new_tessellator_state(&self) -> ThreadLocalTessellatorState {
        ThreadLocalTessellatorState {
            tile_request_state: self.tile_request_state.clone(),
            tessellate_result_sender: self.tessellate_channel.0.clone(),
            geometry_index: self.geometry_index.clone(),
        }
    }

    pub fn try_request_tile(
        &mut self,
        coords: &WorldTileCoords,
        layers: &HashSet<String>,
    ) -> Result<(), SendError<TileRequest>> {
        if !self.tile_cache.is_layers_missing(coords, layers) {
            return Ok(());
        }

        if let Ok(mut tile_request_state) = self.tile_request_state.try_lock() {
            if let Some(request_id) = tile_request_state.start_tile_request(TileRequest {
                coords: *coords,
                layers: layers.clone(),
            }) {
                info!("new tile request: {}", &coords);

                // The following snippet can be added instead of the next code block to demonstrate
                // an understanable approach of fetching
                /*#[cfg(target_arch = "wasm32")]
                if let Some(tile_coords) = coords.into_tile(TileAddressingScheme::TMS) {
                    crate::platform::legacy_webworker_fetcher::request_tile(
                        request_id,
                        tile_coords,
                    );
                }*/

                {
                    let client = SourceClient::Http(HttpSourceClient::new());
                    let copied_coords = *coords;

                    let future_fn = move |thread_local_state: ThreadLocalTessellatorState| async move {
                        if let Ok(data) = client.fetch(&copied_coords).await {
                            thread_local_state
                                .process_tile(request_id, data.into_boxed_slice())
                                .unwrap();
                        } else {
                            thread_local_state.tile_unavailable(request_id).unwrap();
                        }
                    };

                    #[cfg(target_arch = "wasm32")]
                    self.schedule_method.schedule(self, future_fn);
                    #[cfg(not(target_arch = "wasm32"))]
                    self.schedule_method.schedule(self, future_fn);
                }
            }
        }

        Ok(())
    }

    pub fn get_tile_cache(&self) -> &TileCache {
        &self.tile_cache
    }
}

#[derive(Default)]
pub struct TileRequestState {
    current_id: TileRequestID,
    pending_tile_requests: HashMap<TileRequestID, TileRequest>,
    pending_coords: HashSet<WorldTileCoords>,
}

impl TileRequestState {
    pub fn new() -> Self {
        Self {
            current_id: 1,
            pending_tile_requests: Default::default(),
            pending_coords: Default::default(),
        }
    }

    pub fn is_tile_request_pending(&self, coords: &WorldTileCoords) -> bool {
        self.pending_coords.contains(coords)
    }

    pub fn start_tile_request(&mut self, tile_request: TileRequest) -> Option<TileRequestID> {
        if self.is_tile_request_pending(&tile_request.coords) {
            return None;
        }

        self.pending_coords.insert(tile_request.coords);
        let id = self.current_id;
        self.pending_tile_requests.insert(id, tile_request);
        self.current_id += 1;
        Some(id)
    }

    pub fn finish_tile_request(&mut self, id: TileRequestID) -> Option<TileRequest> {
        self.pending_tile_requests.remove(&id).map(|request| {
            self.pending_coords.remove(&request.coords);
            request
        })
    }

    pub fn get_tile_request(&self, id: TileRequestID) -> Option<&TileRequest> {
        self.pending_tile_requests.get(&id)
    }
}
