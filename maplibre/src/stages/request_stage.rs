//! Requests tiles which are currently in view

use crate::coords::ZoomLevel;
use crate::io::apc::AsyncProcedureCall;
use crate::io::pipeline::PipelineContext;
use crate::io::pipeline::Processable;
use crate::io::tile_pipelines::build_vector_tile_pipeline;
use crate::platform::http_client::ReqwestHttpClient;
use crate::stages::HeadedPipelineProcessor;
use crate::{
    context::MapContext,
    coords::{ViewRegion, WorldTileCoords},
    error::Error,
    io::{
        source_client::{HttpSourceClient, SourceClient},
        tile_repository::TileRepository,
        TileRequest,
    },
    schedule::Stage,
    Environment, HttpClient, Scheduler, Style,
};
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;
use std::process::Output;
use std::rc::Rc;
use std::str::FromStr;

pub struct RequestStage<E: Environment> {
    apc: Rc<E::AsyncProcedureCall>,
    http_source_client: HttpSourceClient<E::HttpClient>,
}

impl<E: Environment> RequestStage<E> {
    pub fn new(
        http_source_client: HttpSourceClient<E::HttpClient>,
        apc: Rc<E::AsyncProcedureCall>,
    ) -> Self {
        Self {
            apc,
            http_source_client,
        }
    }
}

impl<E: Environment> Stage for RequestStage<E> {
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

        if view_state.camera.did_change(0.05) || view_state.zoom.did_change(0.05) {
            if let Some(view_region) = &view_region {
                // FIXME: We also need to request tiles from layers above if we are over the maximum zoom level
                self.request_tiles_in_view(tile_repository, style, view_region);
            }
        }

        view_state.camera.update_reference();
        view_state.zoom.update_reference();
    }
}

pub fn schedule<E: Environment>(
    input: TileRequest,
    context: <E::AsyncProcedureCall as AsyncProcedureCall<E::Transferables>>::Context,
) -> Pin<Box<(dyn Future<Output = ()> + Send + 'static)>> {
    Box::pin(async move {
        let coords = input.coords;
        let client = SourceClient::Http(HttpSourceClient::new(ReqwestHttpClient::new(None)));
        let request_id = 0;

        match client.fetch(&coords).await {
            Ok(data) => {
                let data = data.into_boxed_slice();

                let mut pipeline_context =
                    PipelineContext::new(HeadedPipelineProcessor::<E> { context });
                let pipeline = build_vector_tile_pipeline();
                pipeline.process((input, data), &mut pipeline_context);
            }
            Err(e) => {
                log::error!("{:?}", &e);
                //state.tile_unavailable(&coords, request_id).unwrap()
            }
        }
    })
}

impl<E: Environment> RequestStage<E> {
    /// Request tiles which are currently in view.
    #[tracing::instrument(skip_all)]
    fn request_tiles_in_view(
        &self,
        tile_repository: &mut TileRepository,
        style: &Style,
        view_region: &ViewRegion,
    ) {
        let source_layers: HashSet<String> = style
            .layers
            .iter()
            .filter_map(|layer| layer.source_layer.clone())
            .collect();

        for coords in view_region.iter() {
            if coords.build_quad_key().is_some() {
                // TODO: Make tesselation depend on style?
                self.request_tile(tile_repository, &coords, &source_layers)
                    .unwrap();
            }
        }
    }

    fn request_tile(
        &self,
        tile_repository: &mut TileRepository,
        coords: &WorldTileCoords,
        layers: &HashSet<String>,
    ) -> Result<(), Error> {
        /*        if !tile_repository.is_layers_missing(coords, layers) {
            return Ok(false);
        }*/

        if tile_repository.needs_fetching(&coords) {
            tile_repository.create_tile(coords);

            tracing::info!("new tile request: {}", &coords);
            self.apc.schedule(
                TileRequest {
                    coords: *coords,
                    layers: layers.clone(),
                },
                schedule::<E>,
            );
        }

        Ok(())
    }
}
