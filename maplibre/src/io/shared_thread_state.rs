//! Shared thread state.

use crate::coords::{TileCoords, WorldCoords, WorldTileCoords, Zoom};
use crate::error::Error;
use crate::io::geometry_index::{GeometryIndex, IndexProcessor, IndexedGeometry, TileIndex};
use crate::io::tile_request_state::TileRequestState;
use crate::io::{
    LayerTessellateMessage, TessellateMessage, TileFetchResult, TileRequest, TileRequestID,
    TileTessellateMessage,
};
use crate::tessellation::Tessellated;
use std::collections::HashSet;

use crate::tessellation::zero_tessellator::ZeroTessellator;
use geozero::mvt::tile;
use geozero::GeozeroDatasource;
use prost::Message;
use std::sync::{mpsc, Arc, Mutex};

/// Stores and provides access to the thread safe data shared between the schedulers.
#[derive(Clone)]
pub struct SharedThreadState {
    pub tile_request_state: Arc<Mutex<TileRequestState>>,
    pub message_sender: mpsc::Sender<TessellateMessage>,
    pub geometry_index: Arc<Mutex<GeometryIndex>>,
}

impl SharedThreadState {
    fn get_tile_request(&self, request_id: TileRequestID) -> Option<TileRequest> {
        self.tile_request_state
            .lock()
            .ok()
            .and_then(|tile_request_state| tile_request_state.get_tile_request(request_id).cloned())
    }

    #[tracing::instrument(skip_all)]
    pub fn process_tile(&self, request_id: TileRequestID, data: Box<[u8]>) -> Result<(), Error> {
        if let Some(tile_request) = self.get_tile_request(request_id) {
            let coords = tile_request.coords;

            tracing::info!("parsing tile {} with {}bytes", &coords, data.len());

            let _span_ = tracing::span!(tracing::Level::TRACE, "parse_tile_bytes").entered();

            let mut tile = geozero::mvt::Tile::decode(data.as_ref()).expect("failed to load tile");

            let mut index = IndexProcessor::new();

            for mut layer in &mut tile.layers {
                let cloned_layer = layer.clone();
                let layer_name: &str = &cloned_layer.name;
                if !tile_request.layers.contains(layer_name) {
                    continue;
                }

                tracing::info!("layer {} at {} ready", layer_name, &coords);

                let mut tessellator = ZeroTessellator::default();
                if let Err(e) = layer.process(&mut tessellator) {
                    self.message_sender.send(TessellateMessage::Layer(
                        LayerTessellateMessage::UnavailableLayer {
                            coords,
                            layer_name: layer_name.to_owned(),
                        },
                    ))?;

                    tracing::error!(
                        "layer {} at {} tesselation failed {:?}",
                        layer_name,
                        &coords,
                        e
                    );
                } else {
                    self.message_sender.send(TessellateMessage::Layer(
                        LayerTessellateMessage::TessellatedLayer {
                            coords,
                            buffer: tessellator.buffer.into(),
                            feature_indices: tessellator.feature_indices,
                            layer_data: cloned_layer,
                        },
                    ))?;
                }

                // TODO
                // layer.process(&mut index).unwrap();
            }

            let available_layers: HashSet<_> = tile
                .layers
                .iter()
                .map(|layer| layer.name.clone())
                .collect::<HashSet<_>>();

            for missing_layer in tile_request.layers.difference(&available_layers) {
                self.message_sender.send(TessellateMessage::Layer(
                    LayerTessellateMessage::UnavailableLayer {
                        coords,
                        layer_name: missing_layer.to_owned(),
                    },
                ))?;

                tracing::info!(
                    "requested layer {} at {} not found in tile",
                    missing_layer,
                    &coords
                );
            }

            tracing::info!("tile tessellated at {} finished", &tile_request.coords);

            self.message_sender
                .send(TessellateMessage::Tile(TileTessellateMessage {
                    request_id,
                    coords: tile_request.coords,
                }))?;

            if let Ok(mut geometry_index) = self.geometry_index.lock() {
                geometry_index.index_tile(
                    &coords,
                    TileIndex::Linear {
                        list: index.get_geometries(),
                    },
                )
            }
        }

        Ok(())
    }

    pub fn tile_unavailable(
        &self,
        coords: &WorldTileCoords,
        request_id: TileRequestID,
    ) -> Result<(), Error> {
        if let Some(tile_request) = self.get_tile_request(request_id) {
            for to_load in &tile_request.layers {
                tracing::warn!("layer {} at {} unavailable", to_load, coords);
                self.message_sender.send(TessellateMessage::Layer(
                    LayerTessellateMessage::UnavailableLayer {
                        coords: tile_request.coords,
                        layer_name: to_load.to_string(),
                    },
                ))?;
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub fn query_point(
        &self,
        world_coords: &WorldCoords,
        z: u8,
        zoom: Zoom,
    ) -> Option<Vec<IndexedGeometry<f64>>> {
        if let Ok(geometry_index) = self.geometry_index.lock() {
            geometry_index
                .query_point(world_coords, z, zoom)
                .map(|geometries| {
                    geometries
                        .iter()
                        .cloned()
                        .cloned()
                        .collect::<Vec<IndexedGeometry<f64>>>()
                })
        } else {
            unimplemented!()
        }
    }
}
