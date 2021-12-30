use std::collections::VecDeque;
use std::io::Cursor;
use std::sync::{Arc, Condvar, Mutex};

use log::info;
use lyon::tessellation::VertexBuffers;

use vector_tile::parse_tile_reader;
use vector_tile::tile::Tile;

use crate::io::{static_database, TileCoords};
use crate::render::shader_ffi::GpuVertexUniform;
use crate::tesselation::{IndexDataType, Tesselated};

#[derive(Clone)]
pub struct TesselatedTile {
    pub id: u32,
    pub coords: TileCoords,
    pub geometry: VertexBuffers<GpuVertexUniform, IndexDataType>,
}

#[derive(Clone)]
pub struct Cache {
    current_id: u32,
    requests: Arc<WorkQueue<TileCoords>>,
    responses: Arc<WorkQueue<TesselatedTile>>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            current_id: 0,
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
        loop {
            while let Some(coords) = self.requests.pop() {
                if let Some(file) = static_database::get_tile(&coords) {
                    let tile = parse_tile_reader(&mut Cursor::new(file.contents()))
                        .expect("failed to load tile");
                    let mut geometry: VertexBuffers<GpuVertexUniform, IndexDataType> =
                        VertexBuffers::new();

                    tile.tesselate_stroke(&mut geometry, 1);
                    self.responses.push(TesselatedTile {
                        id: self.current_id,
                        coords,
                        geometry,
                    });
                    self.current_id += 1;
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
        } else {
            panic!("locking failed");
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
