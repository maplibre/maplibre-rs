use std::collections::{HashMap, HashSet};

use std::sync::mpsc::{channel, Receiver, SendError, Sender};
use std::sync::{Arc, Mutex};

//use crossbeam_channel::{unbounded as channel, Receiver, RecvError, SendError, Sender};
use log::{info, warn};

use style_spec::source::TileAdressingScheme;
use vector_tile::parse_tile_bytes;

/// Describes through which channels work-requests travel. It describes the flow of work.
use crate::coords::{TileCoords, WorldTileCoords};
use crate::io::tile_cache::TileCache;
use crate::io::{LayerResult, TileRequest, TileRequestID, TileResult};

use crate::tessellation::Tessellated;

pub enum ScheduleMethod {
    #[cfg(not(target_arch = "wasm32"))]
    Tokio(crate::platform::scheduler::TokioScheduleMethod),
    #[cfg(target_arch = "wasm32")]
    WebWorker(crate::platform::scheduler::WebWorkerScheduleMethod),
}

impl ScheduleMethod {
    pub fn schedule_tile_request(
        &self,
        scheduler: &IOScheduler,
        request_id: TileRequestID,
        coords: TileCoords,
    ) {
        match self {
            #[cfg(not(any(
                target_os = "android",
                all(target_arch = "aarch64", not(target_os = "android")),
                target_arch = "wasm32"
            )))]
            ScheduleMethod::Tokio(method) => {
                method.schedule_tile_request(scheduler, request_id, coords)
            }
            #[cfg(target_arch = "wasm32")]
            ScheduleMethod::WebWorker(method) => {
                method.schedule_tile_request(scheduler, request_id, coords)
            }
        }
    }
}

pub struct ThreadLocalTessellatorState {
    tile_request_state: Arc<Mutex<TileRequestState>>,
    layer_result_sender: Sender<LayerResult>,
}

impl ThreadLocalTessellatorState {
    pub fn tessellate_layers(
        &self,
        request_id: TileRequestID,
        data: Box<[u8]>,
    ) -> Result<(), SendError<LayerResult>> {
        if let Ok(tile_request_state) = self.tile_request_state.lock() {
            if let Some(tile_request) = tile_request_state.finish_tile_request(request_id) {
                self.tessellate_layers_with_request(
                    TileResult::Tile {
                        coords: tile_request.coords,
                        data,
                    },
                    tile_request,
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
        tile_result: TileResult,
        tile_request: &TileRequest,
    ) -> Result<(), SendError<LayerResult>> {
        if let TileResult::Tile { data, coords } = tile_result {
            info!("parsing tile {} with {}bytes", &coords, data.len());
            let tile = parse_tile_bytes(&data).expect("failed to load tile");

            for to_load in &tile_request.layers {
                if let Some(layer) = tile
                    .layers()
                    .iter()
                    .find(|layer| to_load.as_str() == layer.name())
                {
                    if let Some((buffer, feature_indices)) = layer.tessellate() {
                        self.layer_result_sender
                            .send(LayerResult::TessellatedLayer {
                                coords,
                                buffer: buffer.into(),
                                feature_indices,
                                layer_data: layer.clone(),
                            })?;
                    }

                    info!("layer {} ready: {}", to_load, &coords);
                } else {
                    self.layer_result_sender
                        .send(LayerResult::UnavailableLayer {
                            coords,
                            layer_name: to_load.to_string(),
                        })?;

                    info!("layer {} not found: {}", to_load, &coords);
                }
            }
        }

        Ok(())
    }
}

pub struct IOScheduler {
    layer_result_sender: Sender<LayerResult>,
    layer_result_receiver: Receiver<LayerResult>,
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

impl Drop for IOScheduler {
    fn drop(&mut self) {
        warn!("WorkerLoop dropped. This should only happen when the application is stopped!");
    }
}

impl IOScheduler {
    pub fn new(schedule_method: ScheduleMethod) -> Self {
        let (layer_result_sender, layer_result_receiver) = channel();
        Self {
            layer_result_sender,
            layer_result_receiver,
            tile_request_state: Arc::new(Mutex::new(TileRequestState::new())),
            tile_cache: TileCache::new(),
            schedule_method,
        }
    }

    pub fn populate_cache(&self) {
        while let Ok(result) = self.layer_result_receiver.try_recv() {
            self.tile_cache.push(result);
        }
    }

    pub fn new_tessellator_state(&self) -> ThreadLocalTessellatorState {
        ThreadLocalTessellatorState {
            tile_request_state: self.tile_request_state.clone(),
            layer_result_sender: self.layer_result_sender.clone(),
        }
    }

    pub fn request_tile(
        &mut self,
        tile_request: TileRequest,
    ) -> Result<(), SendError<TileRequest>> {
        let TileRequest { coords, layers } = &tile_request;

        if let Some(missing_layers) = self
            .tile_cache
            .get_missing_tessellated_layer_names_at(coords, layers.clone())
        {
            if missing_layers.is_empty() {
                return Ok(());
            }

            loop {
                if let Ok(mut tile_request_state) = self.tile_request_state.try_lock() {
                    if let Some(id) = tile_request_state.start_tile_request(tile_request.clone()) {
                        info!("new tile request: {}", &coords);

                        let tile_coords = coords.into_tile(TileAdressingScheme::TMS);
                        self.schedule_method
                            .schedule_tile_request(self, id, tile_coords)
                    }

                    break;
                }
            }

            Ok(())
        } else {
            Ok(())
        }
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

    /*pub fn finish_tile_request(&mut self, id: TileRequestID) -> Option<TileRequest> {
        self.pending_tile_requests.remove(&id).map(|request| {
            self.pending_coords.remove(&request.coords);
            request
        })
    }*/

    pub fn finish_tile_request(&self, id: TileRequestID) -> Option<&TileRequest> {
        self.pending_tile_requests.get(&id)
    }
}
