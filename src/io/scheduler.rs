use std::collections::{HashMap, HashSet};
use std::fmt;

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
    LayerTessellateResult, TileFetchResult, TileIndexResult, TileRequest, TileRequestID,
    TileTessellateResult,
};

use crate::tessellation::Tessellated;

pub enum ScheduleMethod {
    #[cfg(not(target_arch = "wasm32"))]
    Tokio(crate::platform::scheduler::TokioScheduleMethod),
    #[cfg(target_arch = "wasm32")]
    WebWorker(crate::platform::scheduler::WebWorkerScheduleMethod),
    #[cfg(target_arch = "wasm32")]
    WebWorkerPool(crate::platform::scheduler::WebWorkerPoolScheduleMethod),
}

impl Default for ScheduleMethod {
    fn default() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            ScheduleMethod::Tokio(crate::platform::scheduler::TokioScheduleMethod::new(None))
        }
        #[cfg(target_arch = "wasm32")]
        {
            ScheduleMethod::WebWorker(crate::platform::scheduler::WebWorkerScheduleMethod::new())
        }
    }
}

impl ScheduleMethod {
    pub fn schedule_tile_request(
        &self,
        scheduler: &IOScheduler,
        request_id: TileRequestID,
        coords: TileCoords,
    ) {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            ScheduleMethod::Tokio(method) => {
                method.schedule_tile_request(scheduler, request_id, coords)
            }
            #[cfg(target_arch = "wasm32")]
            ScheduleMethod::WebWorker(method) => {
                method.schedule_tile_request(scheduler, request_id, coords)
            }
            #[cfg(target_arch = "wasm32")]
            ScheduleMethod::WebWorkerPool(method) => {
                method.schedule_tile_request(scheduler, request_id, coords)
            }
        }
    }
}

pub struct ThreadLocalTessellatorState {
    tile_request_state: Arc<Mutex<TileRequestState>>,
    tessellate_result_sender: Sender<TileTessellateResult>,
    index_result_sender: Sender<TileIndexResult>,
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
use crate::io::geometry_index::{IndexProcessor, TileIndex};
use prost::Message;

impl ThreadLocalTessellatorState {
    fn get_tile_request(&self, request_id: TileRequestID) -> Option<TileRequest> {
        self.tile_request_state
            .lock()
            .ok()
            .and_then(|tile_request_state| tile_request_state.get_tile_request(request_id).cloned())
    }

    pub fn process_tile(
        &self,
        request_id: TileRequestID,
        data: Box<[u8]>,
    ) -> Result<(), SendError<TileTessellateResult>> {
        if let Some(tile_request) = self.get_tile_request(request_id) {
            let tile_result = TileFetchResult::Tile {
                coords: tile_request.coords,
                data,
            };

            self.tessellate_layers_with_request(&tile_result, &tile_request, request_id)?;

            self.index_geometry(request_id, &tile_result);
        }

        Ok(())
    }

    pub fn tile_unavailable(
        &self,
        request_id: TileRequestID,
    ) -> Result<(), SendError<TileTessellateResult>> {
        if let Some(tile_request) = self.get_tile_request(request_id) {
            let tile_result = TileFetchResult::Unavailable {
                coords: tile_request.coords,
            };
            self.tessellate_layers_with_request(&tile_result, &tile_request, request_id)?;
        }

        Ok(())
    }

    fn index_geometry(&self, request_id: TileRequestID, tile_result: &TileFetchResult) {
        match tile_result {
            TileFetchResult::Tile { data, coords } => {
                let tile: Tile = Tile::decode(data.as_ref()).unwrap();

                let mut processor = IndexProcessor::new();
                for mut layer in tile.layers {
                    layer.process(&mut processor).unwrap();
                }

                self.index_result_sender
                    .send(TileIndexResult {
                        request_id,
                        coords: *coords,
                        /*index: TileIndex::Spatial {
                            tree: processor.build_tree(),
                        },*/
                        index: TileIndex::Linear {
                            list: processor.get_geometries(),
                        },
                    })
                    .unwrap();
            }
            _ => {}
        }
    }

    fn tessellate_layers_with_request(
        &self,
        tile_result: &TileFetchResult,
        tile_request: &TileRequest,
        request_id: TileRequestID,
    ) -> Result<(), SendError<TileTessellateResult>> {
        match tile_result {
            TileFetchResult::Unavailable { coords } => {
                for to_load in &tile_request.layers {
                    self.tessellate_result_sender
                        .send(TileTessellateResult::Layer(
                            LayerTessellateResult::UnavailableLayer {
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
                                    .send(TileTessellateResult::Layer(
                                        LayerTessellateResult::TessellatedLayer {
                                            coords: *coords,
                                            buffer: buffer.into(),
                                            feature_indices,
                                            layer_data: layer.clone(),
                                        },
                                    ))?;
                            }
                            Err(e) => {
                                self.tessellate_result_sender
                                    .send(TileTessellateResult::Layer(
                                        LayerTessellateResult::UnavailableLayer {
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
                            .send(TileTessellateResult::Layer(
                                LayerTessellateResult::UnavailableLayer {
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
            .send(TileTessellateResult::Tile { request_id })?;

        Ok(())
    }
}

pub struct IOScheduler {
    index_channel: (Sender<TileIndexResult>, Receiver<TileIndexResult>),
    tessellate_channel: (Sender<TileTessellateResult>, Receiver<TileTessellateResult>),
    tile_request_state: Arc<Mutex<TileRequestState>>,
    tile_cache: TileCache,
    schedule_method: ScheduleMethod,
}

impl fmt::Debug for IOScheduler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IOScheduler")
    }
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
            index_channel: channel(),
            tessellate_channel: channel(),
            tile_request_state: Arc::new(Mutex::new(TileRequestState::new())),
            tile_cache: TileCache::new(),
            schedule_method,
        }
    }

    pub fn try_populate_cache(&mut self) {
        if let Ok(mut tile_request_state) = self.tile_request_state.try_lock() {
            if let Ok(result) = self.tessellate_channel.1.try_recv() {
                match result {
                    TileTessellateResult::Tile { request_id } => {
                        tile_request_state.finish_tile_request(request_id);
                    }
                    TileTessellateResult::Layer(layer_result) => {
                        self.tile_cache.put_tessellation_result(layer_result);
                    }
                }
            }
        }

        if let Ok(result) = self.index_channel.1.try_recv() {
            self.tile_cache.put_index_result(result);
        }
    }

    pub fn new_tessellator_state(&self) -> ThreadLocalTessellatorState {
        ThreadLocalTessellatorState {
            tile_request_state: self.tile_request_state.clone(),
            tessellate_result_sender: self.tessellate_channel.0.clone(),
            index_result_sender: self.index_channel.0.clone(),
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
            if let Some(id) = tile_request_state.start_tile_request(TileRequest {
                coords: *coords,
                layers: layers.clone(),
            }) {
                if let Some(tile_coords) = coords.into_tile(TileAddressingScheme::TMS) {
                    info!("new tile request: {}", &tile_coords);

                    self.schedule_method
                        .schedule_tile_request(self, id, tile_coords);
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
