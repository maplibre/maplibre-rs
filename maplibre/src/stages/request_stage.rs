//! Requests tiles which are currently in view

use std::{collections::HashSet, rc::Rc};

use crate::{
    context::MapContext,
    coords::{ViewRegion, WorldTileCoords},
    environment::Environment,
    error::Error,
    io::{
        apc::{AsyncProcedureCall, AsyncProcedureFuture, Context, Input, Message},
        pipeline::{PipelineContext, Processable},
        tile_pipelines::build_vector_tile_pipeline,
        tile_repository::TileRepository,
        transferables::{Transferables, UnavailableLayer},
        TileRequest,
    },
    kernel::Kernel,
    schedule::Stage,
    stages::HeadedPipelineProcessor,
    style::Style,
    world::World,
};

pub struct RequestStage<E: Environment> {
    kernel: Rc<Kernel<E>>,
}

impl<E: Environment> RequestStage<E> {
    pub fn new(kernel: Rc<Kernel<E>>) -> Self {
        Self { kernel }
    }
}

impl<E: Environment> Stage for RequestStage<E> {
    fn run(
        &mut self,
        MapContext {
            world:
                World {
                    tile_repository,
                    view_state,
                },
            style,
            ..
        }: &mut MapContext,
    ) {
        let view_region = view_state.create_view_region();

        if view_state.did_camera_change() || view_state.did_zoom_change() {
            if let Some(view_region) = &view_region {
                // FIXME: We also need to request tiles from layers above if we are over the maximum zoom level
                self.request_tiles_in_view(tile_repository, style, view_region);
            }
        }

        view_state.update_references();
    }
}

pub fn schedule<
    E: Environment,
    C: Context<
        <E::AsyncProcedureCall as AsyncProcedureCall<E::HttpClient>>::Transferables,
        E::HttpClient,
    >,
>(
    input: Input,
    context: C,
) -> AsyncProcedureFuture {
    // FIXME: improve input handling
    let input = match input {
        Input::TileRequest(input) => Some(input),
        _ => None,
    }
    .unwrap(); // FIXME (wasm-executor): Remove unwrap

    Box::pin(async move {
        let coords = input.coords;
        let client = context.source_client();

        match client.fetch(&coords).await {
            Ok(data) => {
                let data = data.into_boxed_slice();

                let mut pipeline_context = PipelineContext::new(HeadedPipelineProcessor {
                    context,
                    phantom_t: Default::default(),
                    phantom_hc: Default::default(),
                });
                let pipeline = build_vector_tile_pipeline();
                pipeline.process((input, data), &mut pipeline_context);
            }
            Err(e) => {
                log::error!("{:?}", &e);
                for to_load in &input.layers {
                    tracing::warn!("layer {} at {} unavailable", to_load, coords);
                    // FIXME: Handle result
                    context.send(
                        Message::UnavailableLayer(<<E::AsyncProcedureCall as AsyncProcedureCall<
                            E::HttpClient,
                        >>::Transferables as Transferables>::UnavailableLayer::new(
                            input.coords,
                            to_load.to_string(),
                        )),
                    );
                }
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
                self.request_tile(tile_repository, coords, &source_layers)
                    .unwrap(); // TODO: Remove unwrap
            }
        }
    }

    fn request_tile(
        &self,
        tile_repository: &mut TileRepository,
        coords: WorldTileCoords,
        layers: &HashSet<String>,
    ) -> Result<(), Error> {
        /* TODO: is this still required?
        if !tile_repository.is_layers_missing(coords, layers) {
            return Ok(false);
        }*/

        if tile_repository.has_tile(&coords) {
            tile_repository.create_tile(coords);

            tracing::info!("new tile request: {}", &coords);
            self.kernel.apc().call(
                Input::TileRequest(TileRequest {
                    coords,
                    layers: layers.clone(),
                }),
                schedule::<
                    E,
                    <E::AsyncProcedureCall as AsyncProcedureCall<
                        E::HttpClient,
                    >>::Context,
                >,
            );
        }

        Ok(())
    }
}
