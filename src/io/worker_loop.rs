use std::collections::{BTreeMap, HashSet, VecDeque};

use std::sync::{Arc, Condvar, Mutex};

use log::{error, info};

use crate::coords::TileCoords;
use vector_tile::parse_tile_bytes;
use vector_tile::tile::Layer;

use crate::io::web_tile_fetcher::WebTileFetcher;
use crate::io::{HttpFetcherConfig, TileFetcher};
use crate::render::ShaderVertex;
use crate::tesselation::{IndexDataType, OverAlignedVertexBuffer, Tesselated};
use std::collections::btree_map::Entry;

#[derive(Clone)]
pub enum TesselationResult {
    Unavailable(EmptyLayer),
    TesselatedLayer(TesselatedLayer),
}

impl TesselationResult {
    pub fn get_tile_coords(&self) -> TileCoords {
        match self {
            TesselationResult::Unavailable(result) => result.coords,
            TesselationResult::TesselatedLayer(result) => result.coords,
        }
    }

    pub fn layer_name(&self) -> &str {
        match self {
            TesselationResult::Unavailable(result) => result.layer_name.as_str(),
            TesselationResult::TesselatedLayer(result) => result.layer_data.name(),
        }
    }
}

#[derive(Clone)]
pub struct TesselatedLayer {
    pub coords: TileCoords,
    pub buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
    /// Holds for each feature the count of indices
    pub feature_indices: Vec<u32>,
    pub layer_data: Layer,
}

#[derive(Clone)]
pub struct EmptyLayer {
    pub coords: TileCoords,
    pub layer_name: String,
}

pub struct TileResultStore {
    store: Mutex<BTreeMap<TileCoords, Vec<TesselationResult>>>,
}

impl TileResultStore {
    fn new() -> Self {
        Self {
            store: Mutex::new(BTreeMap::new()),
        }
    }

    fn push(&self, result: TesselationResult) -> bool {
        if let Ok(mut map) = self.store.lock() {
            match map.entry(result.get_tile_coords()) {
                Entry::Vacant(entry) => {
                    entry.insert(vec![result]);
                }
                Entry::Occupied(mut entry) => {
                    entry.get_mut().push(result);
                }
            }
            true
        } else {
            false
        }
    }

    fn get_tesselated_layers_at(
        &self,
        coords: &TileCoords,
        skip_layers: &Vec<String>,
    ) -> Vec<TesselationResult> {
        let mut ret = Vec::new();
        if let Ok(map) = self.store.try_lock() {
            if let Some(results) = map.get(coords) {
                for result in results {
                    if !skip_layers.contains(&result.layer_name().to_string()) {
                        ret.push(result.clone());
                    }
                }
            }
        }

        ret
    }

    fn get_missing_tesselated_layer_names_at(
        &self,
        coords: &TileCoords,
        layers: &Vec<String>,
    ) -> Vec<String> {
        if let Ok(loaded) = self.store.try_lock() {
            if let Some(tesselated_layers) = loaded.get(coords) {
                let mut result = Vec::new();
                for layer in layers {
                    if tesselated_layers
                        .iter()
                        .find(|tesselated_layer| tesselated_layer.layer_name() == layer)
                        .is_none()
                    {
                        result.push(layer.clone());
                    }
                }
                result
            } else {
                layers.clone()
            }
        } else {
            Vec::new()
        }
    }
}

pub struct TileRequest(pub TileCoords, pub Vec<String>);

#[derive(Clone)]
pub struct WorkerLoop {
    requests: Arc<RequestQueue<TileRequest>>,
    tile_result_store: Arc<TileResultStore>,
    pending_tiles: Arc<Mutex<HashSet<TileCoords>>>,
}

impl Drop for WorkerLoop {
    fn drop(&mut self) {
        error!("WorkerLoop dropped. This should only happen when the application is stopped!");
    }
}

impl WorkerLoop {
    pub fn new() -> Self {
        Self {
            requests: Arc::new(RequestQueue::new()),
            tile_result_store: Arc::new(TileResultStore::new()),
            pending_tiles: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn spin_fetch(&self, tile_request: TileRequest) {
        let TileRequest(coords, layers) = &tile_request;

        if let Ok(mut pending_tiles) = self.pending_tiles.try_lock() {
            if pending_tiles.contains(&coords) {
                return;
            }
            pending_tiles.insert(*coords);

            let missing_layers = self
                .tile_result_store
                .get_missing_tesselated_layer_names_at(&coords, &layers);

            if missing_layers.is_empty() {
                return;
            }

            info!("new tile request: {}", &coords);
            self.requests.spin_push(tile_request);
        }
    }

    pub fn get_tesselated_layers_at(
        &self,
        coords: &TileCoords,
        skip_layers: &Vec<String>,
    ) -> Vec<TesselationResult> {
        self.tile_result_store
            .get_tesselated_layers_at(coords, skip_layers)
    }

    pub async fn run_loop(&mut self) {
        let fetcher = WebTileFetcher::new(HttpFetcherConfig {
            cache_path: "/tmp/mapr-cache".to_string(),
        });
        // let fetcher = StaticTileFetcher::new();

        loop {
            while let Some(TileRequest(coords, layers_to_load)) = self.requests.pop() {
                match fetcher.fetch_tile(&coords).await {
                    Ok(data) => {
                        info!("preparing tile {} with {}bytes", &coords, data.len());
                        let tile = parse_tile_bytes(data.as_slice()).expect("failed to load tile");

                        for to_load in layers_to_load {
                            if let Some(layer) = tile
                                .layers()
                                .iter()
                                .find(|layer| to_load.as_str() == layer.name())
                            {
                                if let Some((buffer, feature_indices)) = layer.tesselate() {
                                    self.tile_result_store.push(
                                        TesselationResult::TesselatedLayer(TesselatedLayer {
                                            coords,
                                            buffer: buffer.into(),
                                            feature_indices,
                                            layer_data: layer.clone(),
                                        }),
                                    );
                                }
                            }
                        }
                        info!("layer ready: {:?}", &coords);
                    }
                    Err(err) => {
                        error!("layer failed: {:?}", &err);
                    }
                }
            }
        }
    }
}

struct RequestQueue<T: Send> {
    queue: Mutex<VecDeque<T>>,
    /// Condvar is also supported on WASM
    /// ([see here]( https://github.com/rust-lang/rust/blob/effea9a2a0d501db5722d507690a1a66236933bf/library/std/src/sys/wasm/atomics/condvar.rs))!
    cvar: Condvar,
}

impl<T: Send> RequestQueue<T> {
    fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            cvar: Condvar::new(),
        }
    }

    fn pop(&self) -> Option<T> {
        if let Ok(mut queue) = self.queue.lock() {
            loop {
                match queue.pop_front() {
                    Some(element) => return Some(element),
                    None => queue = self.cvar.wait(queue).unwrap(),
                }
            }
        } else {
            panic!("locking failed");
        }
    }

    fn spin_push(&self, work: T) {
        loop {
            if let Ok(mut queue) = self.queue.try_lock() {
                queue.push_back(work);
                self.cvar.notify_all();
                break;
            }
        }
    }
}
