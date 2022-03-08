/// Describes through which channels work-requests travel. It describes the flow of work.
use crate::coords::TileCoords;
use crate::io::tile_cache::TileCache;
use crate::io::web_tile_fetcher::WebTileFetcher;
use crate::io::{HttpFetcherConfig, TileFetcher};
use crate::render::ShaderVertex;
use crate::tessellation::{IndexDataType, OverAlignedVertexBuffer, Tessellated};
use log::{error, info};
use std::collections::HashSet;
use std::sync::mpsc::{channel, Receiver, SendError, Sender};
use std::sync::Mutex;
use vector_tile::parse_tile_bytes;
use vector_tile::tile::Layer;

pub struct Workflow {
    pub layer_result_receiver: Receiver<LayerResult>,
    pub tile_request_dispatcher: TileRequestDispatcher,
    pub download_tessellate_loop: DownloadTessellateLoop,
}

impl Workflow {
    pub fn create() -> Self {
        let (tile_request_sender, tile_request_receiver) = channel();

        let tile_request_dispatcher = TileRequestDispatcher::new(tile_request_sender);

        let (layer_result_sender, layer_result_receiver) = channel();

        let download_tessellate_loop =
            DownloadTessellateLoop::new(tile_request_receiver, layer_result_sender);

        Self {
            layer_result_receiver,
            tile_request_dispatcher,
            download_tessellate_loop,
        }
    }
}

#[derive(Clone)]
pub enum LayerResult {
    EmptyLayer {
        coords: TileCoords,
        layer_name: String,
    },
    TessellatedLayer {
        coords: TileCoords,
        buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
        /// Holds for each feature the count of indices
        feature_indices: Vec<u32>,
        layer_data: Layer,
    },
}

impl LayerResult {
    pub fn get_tile_coords(&self) -> TileCoords {
        match self {
            LayerResult::EmptyLayer { coords, .. } => *coords,
            LayerResult::TessellatedLayer { coords, .. } => *coords,
        }
    }

    pub fn layer_name(&self) -> &str {
        match self {
            LayerResult::EmptyLayer { layer_name, .. } => layer_name.as_str(),
            LayerResult::TessellatedLayer { layer_data, .. } => layer_data.name(),
        }
    }
}

pub struct TileRequest(pub TileCoords, pub Vec<String>);

pub struct TileRequestDispatcher {
    request_sender: Sender<TileRequest>,
    pending_tiles: HashSet<TileCoords>,
}

impl TileRequestDispatcher {
    pub fn new(request_sender: Sender<TileRequest>) -> Self {
        Self {
            pending_tiles: Default::default(),
            request_sender,
        }
    }

    pub fn request_tile(
        &mut self,
        tile_request: TileRequest,
        tile_cache: &TileCache,
    ) -> Result<(), SendError<TileRequest>> {
        let TileRequest(coords, layers) = &tile_request;

        let missing_layers = tile_cache.get_missing_tessellated_layer_names_at(&coords, &layers);

        if missing_layers.is_empty() {
            return Ok(());
        }

        if self.pending_tiles.contains(&coords) {
            return Ok(());
        }
        self.pending_tiles.insert(*coords);

        info!("new tile request: {}", &coords);
        self.request_sender.send(tile_request)
    }
}

pub struct DownloadTessellateLoop {
    request_receiver: Receiver<TileRequest>,
    result_sender: Sender<LayerResult>,
}

impl DownloadTessellateLoop {
    pub fn new(
        request_receiver: Receiver<TileRequest>,
        result_sender: Sender<LayerResult>,
    ) -> Self {
        Self {
            request_receiver,
            result_sender,
        }
    }

    pub async fn run_loop(&self) {
        let fetcher = WebTileFetcher::new(HttpFetcherConfig {
            cache_path: "/tmp/mapr-cache".to_string(),
        });
        // let fetcher = StaticTileFetcher::new();

        loop {
            // Internally uses Condvar probably: Condvar is also supported on WASM
            // see https://github.com/rust-lang/rust/blob/effea9a2a0d501db5722d507690a1a66236933bf/library/std/src/sys/wasm/atomics/condvar.rs
            if let TileRequest(coords, layers_to_load) = self.request_receiver.recv().unwrap() {
                // TODO remove unwrap
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
                                if let Some((buffer, feature_indices)) = layer.tessellate() {
                                    self.result_sender
                                        .send(LayerResult::TessellatedLayer {
                                            coords,
                                            buffer: buffer.into(),
                                            feature_indices,
                                            layer_data: layer.clone(),
                                        })
                                        .unwrap();
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
