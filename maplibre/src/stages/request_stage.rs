//! Requests tiles which are currently in view

use crate::context::MapContext;
use crate::coords::{ViewRegion, WorldTileCoords};
use crate::error::Error;
use crate::io::shared_thread_state::SharedThreadState;
use crate::io::source_client::SourceClient;
use crate::io::tile_cache::TileCache;
use crate::io::TileRequest;
use crate::schedule::Stage;
use crate::{HTTPClient, ScheduleMethod, Style};
use std::collections::HashSet;

pub struct RequestStage<HC>
where
    HC: HTTPClient,
{
    pub source_client: SourceClient<HC>,
    pub try_failed: bool,
}

impl<HC> RequestStage<HC>
where
    HC: HTTPClient,
{
    pub fn new(source_client: SourceClient<HC>) -> Self {
        Self {
            source_client,
            try_failed: false,
        }
    }
}

impl<HC> Stage for RequestStage<HC>
where
    HC: HTTPClient,
{
    fn run(
        &mut self,
        MapContext {
            view_state,
            style,
            tile_cache,
            scheduler,
            shared_thread_state,
            ..
        }: &mut MapContext,
    ) {
        let visible_level = view_state.visible_level();

        let view_proj = view_state.view_projection();

        let view_region = view_state
            .camera
            .view_region_bounding_box(&view_proj.invert())
            .map(|bounding_box| ViewRegion::new(bounding_box, 0, *view_state.zoom, visible_level));

        if view_state.camera.did_change(0.05) || view_state.zoom.did_change(0.05) || self.try_failed
        {
            if let Some(view_region) = &view_region {
                // FIXME: We also need to request tiles from layers above if we are over the maximum zoom level
                self.try_failed = self.request_tiles_in_view(
                    tile_cache,
                    style,
                    shared_thread_state,
                    scheduler,
                    view_region,
                );
            }
        }

        view_state.camera.update_reference();
        view_state.zoom.update_reference();
    }
}

impl<HC> RequestStage<HC>
where
    HC: HTTPClient,
{
    /// Request tiles which are currently in view.
    #[tracing::instrument(skip_all)]
    fn request_tiles_in_view(
        &self,
        tile_cache: &TileCache,
        style: &Style,
        shared_thread_state: &SharedThreadState,
        scheduler: &Box<dyn ScheduleMethod>,
        view_region: &ViewRegion,
    ) -> bool {
        let mut try_failed = false;
        let source_layers: HashSet<String> = style
            .layers
            .iter()
            .filter_map(|layer| layer.source_layer.clone())
            .collect();

        for coords in view_region.iter() {
            if coords.build_quad_key().is_some() {
                // TODO: Make tesselation depend on style?
                try_failed = self
                    .try_request_tile(
                        tile_cache,
                        shared_thread_state,
                        scheduler,
                        &coords,
                        &source_layers,
                    )
                    .unwrap();
            }
        }
        try_failed
    }

    fn try_request_tile(
        &self,
        tile_cache: &TileCache,
        shared_thread_state: &SharedThreadState,
        scheduler: &Box<dyn ScheduleMethod>,
        coords: &WorldTileCoords,
        layers: &HashSet<String>,
    ) -> Result<bool, Error> {
        if !tile_cache.is_layers_missing(coords, layers) {
            return Ok(false);
        }

        if let Ok(mut tile_request_state) = shared_thread_state.tile_request_state.try_lock() {
            if let Some(request_id) = tile_request_state.start_tile_request(TileRequest {
                coords: *coords,
                layers: layers.clone(),
            }) {
                tracing::info!("new tile request: {}", &coords);

                // The following snippet can be added instead of the next code block to demonstrate
                // an understanable approach of fetching
                /*#[cfg(target_arch = "wasm32")]
                if let Some(tile_coords) = coords.into_tile(TileAddressingScheme::TMS) {
                    crate::platform::legacy_webworker_fetcher::request_tile(
                        request_id,
                        tile_coords,
                    );
                }*/

                let client = self.source_client.clone();
                let coords = *coords;

                scheduler
                    .schedule(
                        shared_thread_state.clone(),
                        Box::new(move |state: SharedThreadState| {
                            Box::pin(async move {
                                match client.fetch(&coords).await {
                                    Ok(data) => state
                                        .process_tile(request_id, data.into_boxed_slice())
                                        .unwrap(),
                                    Err(e) => {
                                        log::error!("{:?}", &e);
                                        state.tile_unavailable(&coords, request_id).unwrap()
                                    }
                                }
                            })
                        }),
                    )
                    .unwrap();
            }

            Ok(false)
        } else {
            Ok(true)
        }
    }
}
