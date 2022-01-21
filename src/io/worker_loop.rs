use std::collections::{HashSet, VecDeque};

use std::sync::{Arc, Condvar, Mutex};

use log::{error, info};

use crate::coords::TileCoords;
use vector_tile::parse_tile_bytes;
use vector_tile::tile::Layer;

use crate::io::web_tile_fetcher::WebTileFetcher;
use crate::io::{HttpFetcherConfig, TileFetcher};
use crate::render::ShaderVertex;
use crate::tesselation::{IndexDataType, OverAlignedVertexBuffer, Tesselated};

pub struct TesselatedLayer {
    pub coords: TileCoords,
    pub buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
    pub feature_vertices: Vec<u32>,
    pub layer_data: Layer,
}

#[derive(Clone)]
pub struct WorkerLoop {
    loaded_coords: Arc<Mutex<HashSet<TileCoords>>>,
    requests: Arc<WorkQueue<TileCoords>>,
    responses: Arc<WorkQueue<TesselatedLayer>>,
}

impl Drop for WorkerLoop {
    fn drop(&mut self) {
        error!("WorkerLoop dropped. This should only happen when the application is stopped!");
    }
}

impl WorkerLoop {
    pub fn new() -> Self {
        Self {
            loaded_coords: Arc::new(Mutex::new(HashSet::new())),
            requests: Arc::new(WorkQueue::new()),
            responses: Arc::new(WorkQueue::new()),
        }
    }

    pub fn try_is_loaded(&self, coords: &TileCoords) -> bool {
        if let Ok(loaded_coords) = self.loaded_coords.try_lock() {
            loaded_coords.contains(coords)
        } else {
            false
        }
    }

    pub fn try_fetch(&mut self, coords: TileCoords) {
        if let Ok(mut loaded_coords) = self.loaded_coords.try_lock() {
            if loaded_coords.contains(&coords) {
                return;
            }
            loaded_coords.insert(coords);
            info!("new tile request: {:?}", &coords);
            self.requests.push(coords);
        }
    }

    pub fn pop_all(&self) -> Vec<TesselatedLayer> {
        self.responses.try_pop_all()
    }

    pub async fn run_loop(&mut self) {
        let fetcher = WebTileFetcher::new(HttpFetcherConfig {
            cache_path: "/tmp/mapr-cache".to_string(),
        });
        // let fetcher = StaticTileFetcher::new();

        loop {
            while let Some(coords) = self.requests.pop() {
                match fetcher.fetch_tile(&coords).await {
                    Ok(data) => {
                        info!("preparing tile {} with {}bytes", &coords, data.len());
                        let tile = parse_tile_bytes(bytemuck::cast_slice(data.as_slice()))
                            .expect("failed to load tile");

                        for layer in tile.layers() {
                            if let Some((buffer, feature_vertices)) = layer.tesselate() {
                                if buffer.indices.is_empty() {
                                    continue;
                                }
                                self.responses.push(TesselatedLayer {
                                    coords,
                                    buffer: buffer.into(),
                                    feature_vertices,
                                    layer_data: layer.clone(),
                                });
                            }
                        }

                        info!("tile ready: {:?}", &coords);
                    }
                    Err(err) => {
                        error!("tile failed: {:?}", &err);
                    }
                }
            }
        }
    }
}

struct WorkQueue<T: Send> {
    inner: Mutex<VecDeque<T>>,
    /// Condvar is also supported on WASM
    /// ([see here]( https://github.com/rust-lang/rust/blob/effea9a2a0d501db5722d507690a1a66236933bf/library/std/src/sys/wasm/atomics/condvar.rs))!
    cvar: Condvar,
}

impl<T: Send> WorkQueue<T> {
    fn new() -> Self {
        Self {
            inner: Mutex::new(VecDeque::new()),
            cvar: Condvar::new(),
        }
    }

    fn try_pop_all(&self) -> Vec<T> {
        let mut result = Vec::new();
        if let Ok(mut queue) = self.inner.try_lock() {
            while let Some(element) = queue.pop_front() {
                result.push(element);
            }
        }
        result
    }

    fn pop(&self) -> Option<T> {
        if let Ok(mut queue) = self.inner.lock() {
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

    fn push(&self, work: T) -> usize {
        if let Ok(mut queue) = self.inner.lock() {
            queue.push_back(work);
            self.cvar.notify_all();
            queue.len()
        } else {
            panic!("locking failed");
        }
    }
}
