//! Requests tiles which are currently in view

use std::{borrow::Cow, collections::HashSet, rc::Rc};

use log::info;

use crate::{
    context::MapContext,
    coords::ViewRegion,
    ecs::{system::System, tiles::Tiles},
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
        transferables::{LayerUnavailable, Transferables},
    },
    kernel::Kernel,
    raster::RasterLayersDataComponent,
    style::Style,
    vector::{VectorLayersDataComponent, VectorLayersIndicesComponent},
};

pub struct RequestSystem<E: Environment> {
    kernel: Rc<Kernel<E>>,
}

impl<E: Environment> RequestSystem<E> {
    pub fn new(kernel: &Rc<Kernel<E>>) -> Self {
        Self {
            kernel: kernel.clone(),
        }
    }
}

impl<E: Environment> System for RequestSystem<E> {
    fn name(&self) -> Cow<'static, str> {
        "request".into()
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
        let tiles = &mut world.tiles;
        let view_state = &mut world.view_state;
        let view_region = view_state.create_view_region();

        if view_state.did_camera_change() || view_state.did_zoom_change() {
            if let Some(view_region) = &view_region {
                // TODO: We also need to request tiles from layers above if we are over the maximum zoom level
                self.request_tiles_in_view(tiles, style, view_region);
            }
        }

        view_state.update_references();
    }
}

impl<E: Environment> RequestSystem<E> {
    /// Request tiles which are currently in view.
    fn request_tiles_in_view(&self, tiles: &mut Tiles, style: &Style, view_region: &ViewRegion) {
        for coords in view_region.iter() {
            if coords.build_quad_key().is_none() {
                continue;
            }

            // TODO: Make tesselation depend on style? So maybe we need to request even if it exists
            if tiles.exists(coords) {
                continue;
            }

            tiles
                .spawn_mut(coords)
                .unwrap()
                .insert(VectorLayersDataComponent::default())
                .insert(VectorLayersIndicesComponent::default())
                .insert(RasterLayersDataComponent::default());

            tracing::event!(tracing::Level::ERROR, %coords, "tile request started: {}", &coords);
            log::info!("tile request started: {}", &coords);

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

pub fn schedule_tile_request<E: Environment, C: Context<E::HttpClient>>(
    input: Input,
    context: C,
) -> AsyncProcedureFuture {
    type Type<E: Environment> =
        <E::AsyncProcedureCall as AsyncProcedureCall<E::HttpClient>>::Transferables;

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
                        PipelineContext::new(HeadedPipelineProcessor::<
                            Type<E>,
                            <E as Environment>::HttpClient,
                            C,
                        >::new(context));
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
                        context
                            .send(<Type<E> as Transferables>::LayerUnavailable::build_from(
                                coords,
                                to_load.to_string(),
                            ))
                            .map_err(ProcedureError::Send)?;
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
                        PipelineContext::new(HeadedPipelineProcessor::<
                            Type<E>,
                            <E as Environment>::HttpClient,
                            C,
                        >::new(context));

                    build_raster_tile_pipeline()
                        .process((RasterTileRequest { coords }, data), &mut pipeline_context)
                        .map_err(|e| ProcedureError::Execution(Box::new(e)))?;
                }
                Err(e) => {
                    log::error!("{:?}", &e);

                    context
                        .send(<Type<E> as Transferables>::LayerUnavailable::build_from(
                            coords,
                            "raster".to_string(),
                        ))
                        .map_err(ProcedureError::Send)?;
                }
            }
        }

        Ok(())
    })
}
