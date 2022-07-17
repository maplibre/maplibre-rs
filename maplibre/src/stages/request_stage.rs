//! Requests tiles which are currently in view

use std::collections::HashSet;

use crate::{
    context::MapContext,
    coords::{ViewRegion, WorldTileCoords},
    error::Error,
    io::{
        source_client::{HttpSourceClient, SourceClient},
        source_type::{RasterSource, Source, TessellateSource},
        tile_repository::TileRepository,
        TileRequest,
    },
    schedule::Stage,
    stages::SharedThreadState,
    HttpClient, ScheduleMethod, Scheduler, Style,
};

pub struct RequestStage<SM, HC>
where
    SM: ScheduleMethod,
    HC: HttpClient,
{
    shared_thread_state: SharedThreadState,
    scheduler: Scheduler<SM>,
    http_source_client: HttpSourceClient<HC>,
    try_failed: bool,
}

impl<SM, HC> RequestStage<SM, HC>
where
    SM: ScheduleMethod,
    HC: HttpClient,
{
    pub fn new(
        shared_thread_state: SharedThreadState,
        http_source_client: HttpSourceClient<HC>,
        scheduler: Scheduler<SM>,
    ) -> Self {
        Self {
            shared_thread_state,
            scheduler,
            http_source_client,
            try_failed: false,
        }
    }
}

impl<SM, HC> Stage for RequestStage<SM, HC>
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
        let view_region = view_state.create_view_region();

        if view_state.camera.did_change(0.05) || view_state.zoom.did_change(0.05) || self.try_failed
        {
            if let Some(view_region) = &view_region {
                // FIXME: We also need to request tiles from layers above if we are over the maximum zoom level
                self.try_failed = self.request_tiles_in_view(tile_repository, style, view_region);
            }
        }

        view_state.camera.update_reference();
        view_state.zoom.update_reference();
    }
}

impl<SM, HC> RequestStage<SM, HC>
where
    SM: ScheduleMethod,
    HC: HttpClient,
{
    /// Request tiles which are currently in view.
    #[tracing::instrument(skip_all)]
    fn request_tiles_in_view(
        &self,
        tile_repository: &TileRepository,
        style: &Style,
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
                    .try_request_tile(tile_repository, &coords, &source_layers)
                    .unwrap();
            }
        }
        try_failed
    }

    fn try_request_tile(
        &self,
        tile_repository: &TileRepository,
        coords: &WorldTileCoords,
        layers: &HashSet<String>,
    ) -> Result<bool, Error> {
        let missed_sources = tile_repository.missed_layer(coords, layers);
        if missed_sources.is_empty() {
            return Ok(false);
        }

        if let Ok(mut tile_request_state) = self.shared_thread_state.tile_request_state.try_lock() {
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

                let raster = "raster".to_string();
                for source in missed_sources {
                    match source {
                        raster => {
                            let raster_source = RasterSource::default();
                            raster_source.load(
                                self.http_source_client.clone(),
                                &self.scheduler,
                                self.shared_thread_state.clone(),
                                coords,
                                request_id,
                            );
                        }
                        _ => {
                            let tessellate_source = TessellateSource::default();
                            tessellate_source.load(
                                self.http_source_client.clone(),
                                &self.scheduler,
                                self.shared_thread_state.clone(),
                                coords,
                                request_id,
                            );
                        }
                    }
                }
            }

            Ok(false)
        } else {
            Ok(true)
        }
    }
}
