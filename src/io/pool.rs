use crate::io::{static_database, TileCoords};
use crate::render::shader_ffi::GpuVertexUniform;
use crate::tesselation::{IndexDataType, Tesselated};
use lyon::tessellation::VertexBuffers;
use std::collections::VecDeque;
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use vector_tile::parse_tile_reader;
use vector_tile::tile::Tile;

#[derive(Clone)]
pub struct TesselatedTile {
    coords: TileCoords,
    geometry: VertexBuffers<GpuVertexUniform, IndexDataType>,
}

#[derive(Clone)]
pub struct Pool {
    requests: WorkQueue<TileCoords>,
    responses: WorkQueue<TesselatedTile>,
}

impl Pool {
    pub fn new() -> Self {
        Self {
            requests: WorkQueue::new(),
            responses: WorkQueue::new(),
        }
    }

    pub fn fetch(&self, coords: TileCoords) {
        self.requests.push(coords);
    }

    pub fn get_available(&self) -> Vec<TesselatedTile> {
        self.responses.pop_all()
    }

    pub fn run_loop(&self) {
        while let Some(coords) = self.requests.pop() {
            if let Some(file) = static_database::get_tile(&coords) {
                let tile = parse_tile_reader(&mut Cursor::new(file.contents()))
                    .expect("failed to load tile");
                let mut geometry: VertexBuffers<GpuVertexUniform, IndexDataType> =
                    VertexBuffers::new();

                let (_tile_stroke_range, _tile_fill_range) = {
                    (
                        tile.tesselate_stroke(&mut geometry, coords.hash()),
                        tile.tesselate_fill(&mut geometry, coords.hash()),
                    )
                };
                self.responses.push(TesselatedTile { coords, geometry });
            }
        }
    }
}

#[derive(Clone)]
struct WorkQueue<T: Send> {
    inner: Arc<Mutex<VecDeque<T>>>,
}

impl<T: Send> WorkQueue<T> {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    fn pop_all(&self) -> Vec<T> {
        let mut result = Vec::new();
        if let Ok(mut queue) = self.inner.lock() {
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
            queue.pop_front()
        } else {
            panic!("locking failed");
        }
    }

    fn push(&self, work: T) -> usize {
        if let Ok(mut queue) = self.inner.lock() {
            queue.push_back(work);
            queue.len()
        } else {
            panic!("locking failed");
        }
    }
}
