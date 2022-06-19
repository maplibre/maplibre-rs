//! Requests tiles which are currently in view

use crate::context::MapContext;
use crate::coords::{ViewRegion, WorldTileCoords};
use crate::io::source_client::{HttpSourceClient, SourceClient};
use crate::schedule::Stage;
use crate::stages::message::{
    LayerTessellateMessage, MessageReceiver, MessageSender, TessellateMessage,
    TileTessellateMessage,
};
use crate::tile::tile_parser::TileParser;
use crate::tile::tile_repository::{StoredLayer};
use crate::tile::tile_tessellator::TileTessellator;
use crate::{HttpClient, ScheduleMethod, Scheduler, Style};
use geozero::mvt::Tile;
use std::sync::mpsc;

pub struct FetchStage<SM, HC>
where
    SM: ScheduleMethod,
    HC: HttpClient,
{
    message_sender: MessageSender,
    message_receiver: MessageReceiver,
    scheduler: Scheduler<SM>,
    http_source_client: HttpSourceClient<HC>,
}

impl<SM, HC> FetchStage<SM, HC>
where
    SM: ScheduleMethod,
    HC: HttpClient,
{
    pub fn new(http_source_client: HttpSourceClient<HC>, scheduler: Scheduler<SM>) -> Self {
        let (message_sender, message_receiver): (MessageSender, MessageReceiver) = mpsc::channel();
        Self {
            message_sender,
            message_receiver,
            scheduler,
            http_source_client,
        }
    }
}

impl<SM, HC> Stage for FetchStage<SM, HC>
where
    SM: ScheduleMethod,
    HC: HttpClient,
{
    fn run(
        &mut self,
        MapContext {
            view_state,
            style,
            tile_repository,
            ..
        }: &mut MapContext,
    ) {
        // Only fetch if the camera has moved
        if view_state.camera.did_change(0.05) || view_state.zoom.did_change(0.05) {
            // Make a projection to find the view region
            let visible_level = view_state.visible_level();
            let view_proj = view_state.view_projection();
            let view_region = view_state
                .camera
                .view_region_bounding_box(&view_proj.invert())
                .map(|bounding_box| {
                    ViewRegion::new(bounding_box, 0, *view_state.zoom, visible_level)
                });

            // Iterate tiles in view_region and asynchronously fetch and process tiles
            if let Some(view_region) = &view_region {
                for coords in view_region.iter() {
                    if coords.build_quad_key().is_some() {
                        if !tile_repository.needs_fetching(&coords) {
                            continue;
                        }

                        tile_repository.create_tile(&coords);

                        self.fetch_tile(&style, &coords);
                    }
                }
            }
        }

        // Receive all tessellations messages
        while let Ok(result) = self.message_receiver.try_recv() {
            match result {
                TessellateMessage::Layer(layer_result) => {
                    let layer: StoredLayer = layer_result.into();
                    tracing::info!(
                        "Layer {} at {} reached main thread",
                        layer.layer_name(),
                        layer.get_coords()
                    );
                    tile_repository.put_tessellated_layer(layer);
                }
                TessellateMessage::Tile(TileTessellateMessage { coords, success }) => {
                    tracing::info!("Tile at {} reached main thread", coords);
                    if success {
                        tile_repository.success(&coords);
                    } else {
                        tile_repository.fail(&coords);
                    }
                }
            }
        }

        view_state.camera.update_reference();
        view_state.zoom.update_reference();
    }
}

impl<SM, HC> FetchStage<SM, HC>
where
    HC: HttpClient,
    SM: ScheduleMethod,
{
    fn fetch_tile(&mut self, style: &Style, coords: &WorldTileCoords) {
        // Fetch the tile and process it
        let client = SourceClient::Http(self.http_source_client.clone());
        let style = style.clone();
        let coords = coords.clone();
        let message_sender = self.message_sender.clone();
        self.scheduler
            .schedule_method()
            .schedule(Box::new(move || {
                Box::pin(async move {
                    match client.fetch(&coords).await {
                        Ok(data) => {
                            let mut tile = TileParser::parse(data.into_boxed_slice());
                            Self::tessellate_tile(&style, coords, tile, &message_sender);
                        }
                        Err(e) => {
                            tracing::error!("{:?}", &e);
                            message_sender
                                .send(TessellateMessage::Tile(TileTessellateMessage {
                                    coords: coords,
                                    success: false,
                                }))
                                .unwrap();
                        }
                    }
                })
            }))
            .unwrap();
    }

    fn tessellate_tile(
        style: &Style,
        coords: WorldTileCoords,
        mut tile: Tile,
        message_sender: &mpsc::Sender<TessellateMessage>,
    ) {
        for mut layer in tile.layers {
            if !style.layers.iter().any(|style_layer| {
                style_layer
                    .source_layer
                    .as_ref()
                    .map_or(false, |layer_name| *layer_name == layer.name)
            }) {
                continue;
            }

            tracing::info!("layer {} at {} ready", &layer.name, coords);

            match TileTessellator::tessellate_layer(&mut layer, style) {
                Err(e) => {
                    tracing::error!(
                        "layer {} at {} tesselation failed {:?}",
                        layer.name,
                        &coords,
                        e
                    );
                }
                Ok((vertex_buffer, feature_indices)) => {
                    tracing::info!("layer {} at {} tesselation success", &layer.name, &coords);
                    message_sender
                        .send(TessellateMessage::Layer(
                            LayerTessellateMessage::TessellatedLayer {
                                coords,
                                buffer: vertex_buffer,
                                feature_indices,
                                layer_data: layer,
                            },
                        ))
                        .unwrap();
                }
            }
        }

        // TODO : Do we need unavailable layer?
        /*let available_layers: HashSet<_> = tile
            .layers
            .iter()
            .map(|layer| layer.name.clone())
            .collect::<HashSet<_>>();

        for missing_layer in tile_request.layers.difference(&available_layers) {
            tracing::info!(
                            "requested layer {} at {} not found in tile",
                            missing_layer,
                            &coords
                        );
        }*/

        tracing::info!("Tile at {} tesselation done", &coords);
        message_sender
            .send(TessellateMessage::Tile(TileTessellateMessage {
                coords,
                success: true,
            }))
            .unwrap();
    }
}
