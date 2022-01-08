use std::collections::VecDeque;
use std::io::Cursor;
use std::ops::Range;
use std::sync::{Arc, Condvar, Mutex};

use log::{error, info};
use lyon::tessellation::VertexBuffers;

use crate::coords::TileCoords;
use vector_tile::parse_tile_bytes;
use vector_tile::tile::Tile;

use crate::io::static_database;
use crate::render::shader_ffi::GpuVertexUniform;
use crate::tesselation::{IndexDataType, OverAlignedVertexBuffer, Tesselated};

#[derive(Clone)]
pub struct TesselatedTile {
    pub id: u32,
    pub coords: TileCoords,
    pub over_aligned: OverAlignedVertexBuffer<GpuVertexUniform, IndexDataType>,
}

#[derive(Clone)]
pub struct Cache {
    requests: Arc<WorkQueue<TileCoords>>,
    responses: Arc<WorkQueue<TesselatedTile>>,
}

impl Drop for Cache {
    fn drop(&mut self) {
        error!("Cache dropped, even though it should never drop!");
    }
}

impl Cache {
    pub fn new() -> Self {
        Self {
            requests: Arc::new(WorkQueue::new()),
            responses: Arc::new(WorkQueue::new()),
        }
    }

    pub fn fetch(&self, coords: TileCoords) {
        info!("new tile request: {:?}", &coords);
        self.requests.push(coords);
    }

    pub fn pop_all(&self) -> Vec<TesselatedTile> {
        self.responses.try_pop_all()
    }

    pub fn run_loop(&mut self) {
        let mut current_id = 0;
        loop {
            while let Some(coords) = self.requests.pop() {
                if let Some(file) = static_database::get_tile(&coords) {
                    info!(
                        "preparing tile {} with {}bytes",
                        &coords,
                        file.contents().len()
                    );
                    let tile = parse_tile_bytes(file.contents()).expect("failed to load tile");

                    let buffer = tile.tesselate_stroke();
                    self.responses.push(TesselatedTile {
                        id: current_id,
                        coords,
                        over_aligned: buffer.into(),
                    });
                    current_id += 1;
                    info!("tile ready: {:?}", &coords);
                } else {
                    info!("tile failed: {:?}", &coords);
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

    fn try_pop(&self) -> Option<T> {
        let mut queue = self.inner.try_lock().ok()?;
        queue.pop_front()
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
