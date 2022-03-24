use std::collections::{HashMap, HashSet};

use log::{error, info};
use std::sync::mpsc::{channel, Receiver, SendError, Sender};
use std::sync::{Arc, Mutex};

use style_spec::source::TileAddressingScheme;
use vector_tile::parse_tile_bytes;

/// Describes through which channels work-requests travel. It describes the flow of work.
use crate::coords::{TileCoords, WorldTileCoords};
use crate::io::tile_cache::TileCache;
use crate::io::{LayerResult, TileFetchResult, TileRequest, TileRequestID, TileTessellateResult};

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
    layer_result_sender: Sender<TileTessellateResult>,
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
    pub fn tessellate_layers(
        &self,
        request_id: TileRequestID,
        data: Box<[u8]>,
    ) -> Result<(), SendError<TileTessellateResult>> {
        if let Ok(tile_request_state) = self.tile_request_state.lock() {
            if let Some(tile_request) = tile_request_state.get_tile_request(request_id) {
                self.tessellate_layers_with_request(
                    TileFetchResult::Tile {
                        coords: tile_request.coords,
                        data,
                    },
                    &tile_request,
                    request_id,
                )
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn tessellate_layers_with_request(
        &self,
        tile_result: TileFetchResult,
        tile_request: &TileRequest,
        request_id: TileRequestID,
    ) -> Result<(), SendError<TileTessellateResult>> {
        if let TileFetchResult::Tile { data, coords } = tile_result {
            info!("parsing tile {} with {}bytes", &coords, data.len());
            let tile = parse_tile_bytes(&data).expect("failed to load tile");

            for to_load in &tile_request.layers {
                if let Some(layer) = tile
                    .layers()
                    .iter()
                    .find(|layer| to_load.as_str() == layer.name())
                {
                    match layer.tessellate() {
                        Ok((buffer, feature_indices)) => {
                            self.layer_result_sender.send(TileTessellateResult::Layer(
                                LayerResult::TessellatedLayer {
                                    coords,
                                    buffer: buffer.into(),
                                    feature_indices,
                                    layer_data: layer.clone(),
                                },
                            ))?;
                        }
                        Err(e) => {
                            self.layer_result_sender.send(TileTessellateResult::Layer(
                                LayerResult::UnavailableLayer {
                                    coords,
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
                    self.layer_result_sender.send(TileTessellateResult::Layer(
                        LayerResult::UnavailableLayer {
                            coords,
                            layer_name: to_load.to_string(),
                        },
                    ))?;

                    info!("layer {} not found: {}", to_load, &coords);
                }
            }
            self.layer_result_sender
                .send(TileTessellateResult::Tile { request_id })?;
        }

        Ok(())
    }
}

pub struct IOScheduler {
    result_sender: Sender<TileTessellateResult>,
    result_receiver: Receiver<TileTessellateResult>,
    tile_request_state: Arc<Mutex<TileRequestState>>,
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
        let (result_sender, result_receiver) = channel();
        Self {
            result_sender,
            result_receiver,
            tile_request_state: Arc::new(Mutex::new(TileRequestState::new())),
            tile_cache: TileCache::new(),
            schedule_method,
        }
    }

    pub fn try_populate_cache(&mut self) {
        if let Ok(result) = self.result_receiver.try_recv() {
            match result {
                TileTessellateResult::Tile { request_id } => loop {
                    if let Ok(mut tile_request_state) = self.tile_request_state.try_lock() {
                        tile_request_state.finish_tile_request(request_id);
                        break;
                    }
                },
                TileTessellateResult::Layer(layer_result) => {
                    self.tile_cache.push(layer_result);
                }
            }
        }
    }

    pub fn new_tessellator_state(&self) -> ThreadLocalTessellatorState {
        ThreadLocalTessellatorState {
            tile_request_state: self.tile_request_state.clone(),
            layer_result_sender: self.result_sender.clone(),
        }
    }

    pub fn try_request_tile(
        &mut self,
        tile_request: TileRequest,
    ) -> Result<(), SendError<TileRequest>> {
        let TileRequest { coords, layers } = &tile_request;

        let mut missing_layers = layers.clone();
        self.tile_cache
            .retain_missing_layer_names(coords, &mut missing_layers);

        if missing_layers.is_empty() {
            return Ok(());
        }

        if let Ok(mut tile_request_state) = self.tile_request_state.try_lock() {
            let tile_coords = *coords;

            if let Some(id) = tile_request_state.start_tile_request(tile_request) {
                info!("new tile request: {}", &tile_coords);

                if let Some(tile_coords) = tile_coords.into_tile(TileAddressingScheme::TMS) {
                    self.schedule_method
                        .schedule_tile_request(self, id, tile_coords);
                }
            }
        }

        Ok(())
    }

    pub fn get_tessellated_layers_at(
        &self,
        coords: &WorldTileCoords,
        skip_layers: &HashSet<String>,
    ) -> Vec<LayerResult> {
        self.tile_cache
            .get_tessellated_layers_at(coords, skip_layers)
    }
}

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

    pub fn get_tile_request(&self, id: TileRequestID) -> Option<TileRequest> {
        self.pending_tile_requests.get(&id).cloned()
    }
}
