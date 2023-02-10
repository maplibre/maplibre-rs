//! Requests tiles which are currently in view

use std::{borrow::Cow, collections::HashSet, marker::PhantomData, rc::Rc};

use crate::{
    context::MapContext,
    ecs::system::System,
    environment::Environment,
    io::{
        apc::{AsyncProcedureCall, AsyncProcedureFuture, Context, Input, ProcedureError},
        pipeline::Processable,
        source_type::{SourceType, TessellateSource},
    },
    kernel::Kernel,
    vector::{
        transferables::{LayerUnavailable, Transferables},
        vector_pipeline::{build_vector_tile_pipeline, VectorPipelineProcessor, VectorTileRequest},
        VectorLayersDataComponent, VectorLayersIndicesComponent,
    },
};

pub struct RequestSystem<E: Environment, T> {
    kernel: Rc<Kernel<E>>,
    phantom_t: PhantomData<T>,
}

impl<E: Environment, T> RequestSystem<E, T> {
    pub fn new(kernel: &Rc<Kernel<E>>) -> Self {
        Self {
            kernel: kernel.clone(),
            phantom_t: Default::default(),
        }
    }
}

impl<E: Environment, T: Transferables> System for RequestSystem<E, T> {
    fn name(&self) -> Cow<'static, str> {
        "vector_request".into()
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

                for coords in view_region.iter() {
                    if coords.build_quad_key().is_none() {
                        continue;
                    }

                    // TODO: Make tesselation depend on style? So maybe we need to request even if it exists
                    if world
                        .tiles
                        .query::<(&VectorLayersDataComponent, &VectorLayersIndicesComponent)>(
                            coords,
                        )
                        .is_some()
                    {
                        continue;
                    }

                    world
                        .tiles
                        .spawn_mut(coords)
                        .unwrap()
                        .insert(VectorLayersDataComponent::default())
                        .insert(VectorLayersIndicesComponent::default());

                    tracing::event!(tracing::Level::ERROR, %coords, "tile request started: {}", &coords);
                    log::info!("tile request started: {}", &coords);

                    self.kernel
                        .apc()
                        .call(
                            Input::TileRequest {
                                coords,
                                style: style.clone(), // TODO: Avoid cloning whole style
                            },
                            fetch_vector_apc::<
                                E,
                                T,
                                <E::AsyncProcedureCall as AsyncProcedureCall<E::HttpClient>>::Context,
                            >,
                        )
                        .unwrap(); // TODO: Remove unwrap
                }
            }
        }

        view_state.update_references();
    }
}

pub fn fetch_vector_apc<E: Environment, T: Transferables, C: Context<E::HttpClient>>(
    input: Input,
    context: C,
) -> AsyncProcedureFuture {
    Box::pin(async move {
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

        let client = context.source_client();

        if !fill_layers.is_empty() {
            let context = context.clone();
            let source = SourceType::Tessellate(TessellateSource::default());
            match client.fetch(&coords, &source).await {
                Ok(data) => {
                    let data = data.into_boxed_slice();

                    let mut pipeline_context =
                        VectorPipelineProcessor::<T, <E as Environment>::HttpClient, C>::new(
                            context,
                        );
                    build_vector_tile_pipeline::<T, <E as Environment>::HttpClient, C>()
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
                            .send(<T as Transferables>::LayerUnavailable::build_from(
                                coords,
                                to_load.to_string(),
                            ))
                            .map_err(ProcedureError::Send)?;
                    }
                }
            }
        }

        Ok(())
    })
}
