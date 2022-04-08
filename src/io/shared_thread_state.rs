use crate::coords::{WorldCoords, Zoom};
use crate::error::Error;
use crate::io::geometry_index::{GeometryIndex, IndexProcessor, IndexedGeometry, TileIndex};
use crate::io::tile_request_state::TileRequestState;
use crate::io::{
    LayerTessellateMessage, TessellateMessage, TileFetchResult, TileRequest, TileRequestID,
    TileTessellateMessage,
};
use crate::tessellation::Tessellated;

use std::sync::{mpsc, Arc, Mutex};

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
            let tile_result = TileFetchResult::Tile {
                coords: tile_request.coords,
                data,
            };

            self.tessellate_layers_with_request(&tile_result, &tile_request, request_id)?;
            self.index_geometry(&tile_result);
        }

        Ok(())
    }

    pub fn tile_unavailable(&self, request_id: TileRequestID) -> Result<(), Error> {
        if let Some(tile_request) = self.get_tile_request(request_id) {
            let tile_result = TileFetchResult::Unavailable {
                coords: tile_request.coords,
            };
            self.tessellate_layers_with_request(&tile_result, &tile_request, request_id)?;
        }

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    fn index_geometry(&self, tile_result: &TileFetchResult) {
        match tile_result {
            TileFetchResult::Tile { data, coords } => {
                use geozero::GeozeroDatasource;
                use prost::Message;

                let tile = geozero::mvt::Tile::decode(data.as_ref()).unwrap();

                let mut processor = IndexProcessor::new();
                for mut layer in tile.layers {
                    layer.process(&mut processor).unwrap();
                }

                if let Ok(mut geometry_index) = self.geometry_index.lock() {
                    geometry_index.index_tile(
                        &coords,
                        TileIndex::Linear {
                            list: processor.get_geometries(),
                        },
                    )
                }
            }
            _ => {}
        }
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

    #[tracing::instrument(skip_all)]
    fn tessellate_layers_with_request(
        &self,
        tile_result: &TileFetchResult,
        tile_request: &TileRequest,
        request_id: TileRequestID,
    ) -> Result<(), Error> {
        match tile_result {
            TileFetchResult::Unavailable { coords } => {
                for to_load in &tile_request.layers {
                    tracing::warn!("layer {} at {} unavailable", to_load, &coords);
                    self.message_sender.send(TessellateMessage::Layer(
                        LayerTessellateMessage::UnavailableLayer {
                            coords: *coords,
                            layer_name: to_load.to_string(),
                        },
                    ))?;
                }
            }
            TileFetchResult::Tile { data, coords } => {
                tracing::info!("parsing tile {} with {}bytes", &coords, data.len());

                let tile = {
                    let _span_ =
                        tracing::span!(tracing::Level::TRACE, "parse_tile_bytes").entered();
                    vector_tile::parse_tile_bytes(data).expect("failed to load tile")
                };

                for to_load in &tile_request.layers {
                    if let Some(layer) = tile
                        .layers()
                        .iter()
                        .find(|layer| to_load.as_str() == layer.name())
                    {
                        match layer.tessellate() {
                            Ok((buffer, feature_indices)) => {
                                tracing::info!("layer {} at {} ready", to_load, &coords);
                                self.message_sender.send(TessellateMessage::Layer(
                                    LayerTessellateMessage::TessellatedLayer {
                                        coords: *coords,
                                        buffer: buffer.into(),
                                        feature_indices,
                                        layer_data: layer.clone(),
                                    },
                                ))?;
                            }
                            Err(e) => {
                                self.message_sender.send(TessellateMessage::Layer(
                                    LayerTessellateMessage::UnavailableLayer {
                                        coords: *coords,
                                        layer_name: to_load.to_string(),
                                    },
                                ))?;

                                tracing::error!(
                                    "layer {} at {} tesselation failed {:?}",
                                    to_load,
                                    &coords,
                                    e
                                );
                            }
                        }
                    } else {
                        self.message_sender.send(TessellateMessage::Layer(
                            LayerTessellateMessage::UnavailableLayer {
                                coords: *coords,
                                layer_name: to_load.to_string(),
                            },
                        ))?;

                        tracing::info!(
                            "requested layer {} at {} not found in tile",
                            to_load,
                            &coords
                        );
                    }
                }
            }
        }

        tracing::info!("tile at {} finished", &tile_request.coords);

        self.message_sender
            .send(TessellateMessage::Tile(TileTessellateMessage {
                request_id,
                coords: tile_request.coords,
            }))?;

        Ok(())
    }
}
