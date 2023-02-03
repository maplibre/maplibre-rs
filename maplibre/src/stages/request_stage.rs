//! Requests tiles which are currently in view

use std::{collections::HashSet, rc::Rc};

use log::info;

use crate::{
    context::MapContext,
    coords::{ViewRegion, WorldTileCoords},
    ecs::world::World,
    environment::Environment,
    io::{
        apc::{AsyncProcedureCall, AsyncProcedureFuture, Context, Input, Message, ProcedureError},
        pipeline::{PipelineContext, Processable},
        source_type::{RasterSource, SourceType, TessellateSource},
        tile_pipelines::{
            build_raster_tile_pipeline, build_vector_tile_pipeline, RasterTileRequest,
            VectorTileRequest,
        },
        tile_repository::TileRepository,
        transferables::{LayerUnavailable, Transferables},
    },
    kernel::Kernel,
    schedule::Stage,
    stages::HeadedPipelineProcessor,
    style::Style,
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
                    ..
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
    Box::pin(async move {
        info!("Processing thread: {:?}", std::thread::current().name());

        let Input::TileRequest {coords, style} = input else {
            return Err(ProcedureError::IncompatibleInput)
        };

        let fill_layers: HashSet<String> = style
            .layers
            .iter()
            .filter_map(|layer| {
                if layer.typ == "fill" || layer.typ == "line" {
                    layer.source_layer.clone()
                } else {
                    None
                }
            })
            .collect();

        let raster_layers: HashSet<String> = style
            .layers
            .iter()
            .filter_map(|layer| {
                if layer.typ == "raster" {
                    layer.source_layer.clone()
                } else {
                    None
                }
            })
            .collect();

        let client = context.source_client();

        if !fill_layers.is_empty() {
            let context = context.clone();
            let source = SourceType::Tessellate(TessellateSource::default());
            match client.fetch(&coords, &source).await {
                Ok(data) => {
                    let data = data.into_boxed_slice();

                    let mut pipeline_context =
                        PipelineContext::new(HeadedPipelineProcessor::new(context));
                    build_vector_tile_pipeline()
                        .process(
                            (
                                VectorTileRequest {
                                    coords,
                                    layers: fill_layers,
                                },
                                data,
                            ),
                            &mut pipeline_context,
                        )
                        .map_err(|e| ProcedureError::Execution(Box::new(e)))?;
                }
                Err(e) => {
                    log::error!("{:?}", &e);
                    for to_load in &fill_layers {
                        context.send(
                            Message::LayerUnavailable(<<E::AsyncProcedureCall as AsyncProcedureCall<
                                E::HttpClient,
                            >>::Transferables as Transferables>::LayerUnavailable::build_from(
                                coords,
                                to_load.to_string(),
                            )),
                        ).map_err(ProcedureError::Send)?;
                    }
                }
            }
        }

        if !raster_layers.is_empty() {
            let context = context.clone();
            let source = SourceType::Raster(RasterSource::default());

            match client.fetch(&coords, &source).await {
                Ok(data) => {
                    let data = data.into_boxed_slice();

                    let mut pipeline_context =
                        PipelineContext::new(HeadedPipelineProcessor::new(context));

                    build_raster_tile_pipeline()
                        .process((RasterTileRequest { coords }, data), &mut pipeline_context)
                        .map_err(|e| ProcedureError::Execution(Box::new(e)))?;
                }
                Err(e) => {
                    log::error!("{:?}", &e);

                    context.send(
                            Message::LayerUnavailable(<<E::AsyncProcedureCall as AsyncProcedureCall<
                                E::HttpClient,
                            >>::Transferables as Transferables>::LayerUnavailable::build_from(
                                coords,
                                "raster".to_string(),
                            )),
                        ).map_err(ProcedureError::Send)?;
                }
            }
        }

        Ok(())
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
        for coords in view_region.iter() {
            if coords.build_quad_key().is_some() {
                // TODO: Make tesselation depend on style?
                self.request_tile(tile_repository, coords, &style);
            }
        }
    }

    fn request_tile(
        &self,
        tile_repository: &mut TileRepository,
        coords: WorldTileCoords,
        style: &Style,
    ) {
        if tile_repository.is_tile_pending_or_done(&coords) {
            tile_repository.mark_tile_pending(coords).unwrap(); // TODO: Remove unwrap

            tracing::event!(tracing::Level::ERROR, %coords, "tile request started: {}", &coords);

            self.kernel
                .apc()
                .call(
                    Input::TileRequest {
                        coords,
                        style: style.clone(), // TODO: Avoid cloning whole style
                    },
                    schedule::<
                        E,
                        <E::AsyncProcedureCall as AsyncProcedureCall<E::HttpClient>>::Context,
                    >,
                )
                .unwrap(); // TODO: Remove unwrap
        }
    }
}
