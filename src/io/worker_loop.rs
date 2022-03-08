use std::collections::{BTreeMap, HashSet, VecDeque};

use std::sync::{Arc, Condvar, Mutex};

use log::{error, info};

use crate::coords::TileCoords;
use vector_tile::parse_tile_bytes;
use vector_tile::tile::Layer;

use crate::io::tile_cache::TileCache;
use crate::io::web_tile_fetcher::WebTileFetcher;
use crate::io::workflow::{LayerResult, TileRequest};
use crate::io::{HttpFetcherConfig, TileFetcher};
use crate::render::ShaderVertex;
use crate::tessellation::{IndexDataType, OverAlignedVertexBuffer, Tessellated};
use std::collections::btree_map::Entry;
use std::sync::mpsc::{channel, Receiver, Sender};

/*impl Drop for WorkerLoop {
    fn drop(&mut self) {
        error!("WorkerLoop dropped. This should only happen when the application is stopped!");
    }
}
*/

struct RequestQueue<T: Send> {
    queue: Mutex<VecDeque<T>>,

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
