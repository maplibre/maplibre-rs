/// Describes through which channels work-requests travel. It describes the flow of work.
use crate::coords::WorldTileCoords;
use crate::io::tile_cache::TileCache;
use crate::io::web_tile_fetcher::WebTileFetcher;
use crate::io::{HttpFetcherConfig, TileFetcher};
use crate::render::ShaderVertex;
use crate::tessellation::{IndexDataType, OverAlignedVertexBuffer, Tessellated};
//use crossbeam_channel::{unbounded as channel, Receiver, RecvError, SendError, Sender};
use log::{error, info, warn};
use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::sync::mpsc::{channel, Receiver, RecvError, SendError, Sender};

use style_spec::source::TileAdressingScheme;
use vector_tile::parse_tile_bytes;
use vector_tile::tile::Layer;

pub struct Workflow {
    layer_result_receiver: Receiver<LayerResult>,
    tile_request_dispatcher: TileRequestDispatcher,
    download_tessellate_loop: Option<DownloadTessellateLoop>,
    tile_cache: TileCache,
}

impl Drop for Workflow {
    fn drop(&mut self) {
        warn!("WorkerLoop dropped. This should only happen when the application is stopped!");
    }
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
            download_tessellate_loop: Some(download_tessellate_loop),
            tile_cache: TileCache::new(),
        }
    }

    pub fn populate_cache(&self) {
        while let Ok(result) = self.layer_result_receiver.try_recv() {
            self.tile_cache.push(result);
        }
    }

    pub fn request_tile(
        &mut self,
        tile_request: TileRequest,
    ) -> Result<(), SendError<TileRequest>> {
        self.tile_request_dispatcher
            .request_tile(tile_request, &self.tile_cache)
    }

    pub fn get_tessellated_layers_at(
        &self,
        coords: &WorldTileCoords,
        skip_layers: &HashSet<String>,
    ) -> Vec<LayerResult> {
        self.tile_cache
            .get_tessellated_layers_at(coords, skip_layers)
    }

    pub fn take_download_loop(&mut self) -> DownloadTessellateLoop {
        self.download_tessellate_loop.take().unwrap()
    }
}

#[derive(Clone)]
pub enum LayerResult {
    UnavailableLayer {
        coords: WorldTileCoords,
        layer_name: String,
    },
    TessellatedLayer {
        coords: WorldTileCoords,
        buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
        /// Holds for each feature the count of indices
        feature_indices: Vec<u32>,
        layer_data: Layer,
    },
}

impl Debug for LayerResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "LayerResult{}", self.get_coords())
    }
}

impl LayerResult {
    pub fn get_coords(&self) -> WorldTileCoords {
        match self {
            LayerResult::UnavailableLayer { coords, .. } => *coords,
            LayerResult::TessellatedLayer { coords, .. } => *coords,
        }
    }

    pub fn layer_name(&self) -> &str {
        match self {
            LayerResult::UnavailableLayer { layer_name, .. } => layer_name.as_str(),
            LayerResult::TessellatedLayer { layer_data, .. } => layer_data.name(),
        }
    }
}

pub struct TileRequest {
    pub coords: WorldTileCoords,
    pub layers: HashSet<String>,
}

pub struct TileRequestDispatcher {
    request_sender: Sender<TileRequest>,
    pending_tiles: HashSet<WorldTileCoords>,
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
        let TileRequest { coords, layers } = &tile_request;

        if let Some(missing_layers) =
            tile_cache.get_missing_tessellated_layer_names_at(coords, layers.clone())
        {
            if missing_layers.is_empty() {
                return Ok(());
            }

            if self.pending_tiles.contains(coords) {
                return Ok(());
            }
            self.pending_tiles.insert(*coords);

            info!("new tile request: {}", &coords);
            self.request_sender.send(tile_request)
        } else {
            Ok(())
        }
    }
}

pub struct DownloadTessellateLoop {
    request_receiver: Receiver<TileRequest>,
    result_sender: Sender<LayerResult>,
}

#[derive(Debug)]
pub enum SendReceiveError<S> {
    Send(SendError<S>),
    Receive(RecvError),
}

impl<S> From<RecvError> for SendReceiveError<S> {
    fn from(e: RecvError) -> Self {
        SendReceiveError::Receive(e)
    }
}

impl<S> From<SendError<S>> for SendReceiveError<S> {
    fn from(e: SendError<S>) -> Self {
        SendReceiveError::Send(e)
    }
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

    pub async fn run_loop(&self) -> Result<(), SendReceiveError<LayerResult>> {
        let fetcher = WebTileFetcher::new(HttpFetcherConfig {
            cache_path: "/tmp/mapr-cache".to_string(),
        });
        // let fetcher = StaticTileFetcher::new();

        loop {
            // Internally uses Condvar probably: Condvar is also supported on WASM
            // see https://github.com/rust-lang/rust/blob/effea9a2a0d501db5722d507690a1a66236933bf/library/std/src/sys/wasm/atomics/condvar.rs
            if let TileRequest {
                coords,
                layers: layers_to_load,
            } = self.request_receiver.recv()?
            {
                let tile_coords = coords.into_tile(TileAdressingScheme::TMS);
                match fetcher.fetch_tile(&tile_coords).await {
                    Ok(data) => {
                        info!("preparing tile {} with {}bytes", &tile_coords, data.len());
                        let tile = parse_tile_bytes(data.as_slice()).expect("failed to load tile1");

                        for to_load in layers_to_load {
                            if let Some(layer) = tile
                                .layers()
                                .iter()
                                .find(|layer| to_load.as_str() == layer.name())
                            {
                                if let Some((buffer, feature_indices)) = layer.tessellate() {
                                    self.result_sender.send(LayerResult::TessellatedLayer {
                                        coords,
                                        buffer: buffer.into(),
                                        feature_indices,
                                        layer_data: layer.clone(),
                                    })?;
                                }

                                info!("layer {} ready: {}", to_load, &coords);
                            } else {
                                self.result_sender.send(LayerResult::UnavailableLayer {
                                    coords,
                                    layer_name: to_load.to_string(),
                                })?;

                                info!("layer {} not found: {}", to_load, &coords);
                            }
                        }
                    }
                    Err(err) => {
                        error!("fetching tile failed: {:?}", &err);
                    }
                }
            }
        }
    }
}
