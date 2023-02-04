//! Requests tiles which are currently in view

use std::{borrow::Cow, collections::HashSet, rc::Rc};

use log::info;

use crate::{
    context::MapContext,
    coords::{ViewRegion, WorldTileCoords},
    ecs::{system::System, world::World},
    environment::Environment,
    io::{
        apc::{
            AsyncProcedureCall, AsyncProcedureFuture, Context, HeadedPipelineProcessor, Input,
            Message, ProcedureError,
        },
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
    style::Style,
};

pub struct RequestSystem<E: Environment> {
    kernel: Rc<Kernel<E>>,
}

impl<E: Environment> System for RequestSystem<E> {
    fn name(&self) -> Cow<'static, str> {
        "populate_world_system".into()
    }

    fn run(
        &mut self,
        MapContext {
            style,
            world,
            renderer,
            ..
        }: &mut MapContext,
    ) {
        let view_state = &mut world.view_state;
        let view_region = view_state.create_view_region();

        if view_state.did_camera_change() || view_state.did_zoom_change() {
            if let Some(view_region) = &view_region {
                // FIXME: We also need to request tiles from layers above if we are over the maximum zoom level
                self.request_tiles_in_view(&mut world.tile_repository, style, view_region);
            }
        }

        view_state.update_references();
    }
}

impl<E: Environment> RequestSystem<E> {
    /// Request tiles which are currently in view.
    fn request_tiles_in_view(
        &self,
        tile_repository: &mut TileRepository,
        style: &Style,
        view_region: &ViewRegion,
    ) {
        for coords in view_region.iter() {
            if !coords.build_quad_key().is_some() {
                continue;
            }
            // TODO: Make tesselation depend on style?
            if !tile_repository.is_tile_pending_or_done(&coords) {
                continue;
            }
            tile_repository.mark_tile_pending(coords).unwrap(); // TODO: Remove unwrap

            tracing::event!(tracing::Level::ERROR, %coords, "tile request started: {}", &coords);

            self.kernel
                .apc()
                .call(
                    Input::TileRequest {
                        coords,
                        style: style.clone(), // TODO: Avoid cloning whole style
                    },
                    schedule_tile_request::<
                        E,
                        <E::AsyncProcedureCall as AsyncProcedureCall<E::HttpClient>>::Context,
                    >,
                )
                .unwrap(); // TODO: Remove unwrap
        }
    }
}

pub fn schedule_tile_request<
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
