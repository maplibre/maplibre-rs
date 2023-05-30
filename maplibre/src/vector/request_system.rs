//! Requests tiles which are currently in view

use std::{borrow::Cow, collections::HashSet, marker::PhantomData, rc::Rc};

use crate::{
    context::MapContext,
    environment::{Environment, OffscreenKernelEnvironment},
    io::{
        apc::{AsyncProcedureCall, AsyncProcedureFuture, Context, Input, ProcedureError},
        source_type::{SourceType, TessellateSource},
    },
    kernel::Kernel,
    style::layer::LayerPaint,
    tcs::system::System,
    vector::{
        process_vector::{process_vector_tile, ProcessVectorContext, VectorTileRequest},
        transferables::{LayerMissing, VectorTransferables},
        VectorLayersDataComponent,
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

impl<E: Environment, T: VectorTransferables> System for RequestSystem<E, T> {
    fn name(&self) -> Cow<'static, str> {
        "vector_request".into()
    }

    fn run(
        &mut self,
        MapContext {
            style,
            view_state,
            world,
            ..
        }: &mut MapContext,
    ) {
        let _tiles = &mut world.tiles;
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
                        .query::<&VectorLayersDataComponent>(coords)
                        .is_some()
                    {
                        continue;
                    }

                    world
                        .tiles
                        .spawn_mut(coords)
                        .unwrap()
                        .insert(VectorLayersDataComponent::default());

                    tracing::event!(tracing::Level::ERROR, %coords, "tile request started: {coords}");
                    log::info!("tile request started: {coords}");

                    self.kernel
                        .apc()
                        .call(
                            Input::TileRequest {
                                coords,
                                style: style.clone(), // TODO: Avoid cloning whole style
                            },
                            fetch_vector_apc::<
                                E::OffscreenKernelEnvironment,
                                T,
                                <E::AsyncProcedureCall as AsyncProcedureCall<
                                    E::OffscreenKernelEnvironment,
                                >>::Context,
                            >,
                        )
                        .unwrap(); // TODO: Remove unwrap
                }
            }
        }

        view_state.update_references();
    }
}

pub fn fetch_vector_apc<
    K: OffscreenKernelEnvironment,
    T: VectorTransferables,
    C: Context + Clone + Send,
>(
    input: Input,
    context: C,
    kernel: K,
) -> AsyncProcedureFuture {
    Box::pin(async move {
        let Input::TileRequest {coords, style} = input else {
            return Err(ProcedureError::IncompatibleInput)
        };

        let fill_layers: HashSet<String> = style
            .layers
            .iter()
            .filter_map(|layer| {
                if matches!(layer.paint, Some(LayerPaint::Fill(_)))
                    || matches!(layer.paint, Some(LayerPaint::Line(_)))
                {
                    layer.source_layer.clone()
                } else {
                    None
                }
            })
            .collect();

        let client = kernel.source_client();

        if !fill_layers.is_empty() {
            let context = context.clone();
            let source = SourceType::Tessellate(TessellateSource::default());
            match client.fetch(&coords, &source).await {
                Ok(data) => {
                    let data = data.into_boxed_slice();

                    let mut pipeline_context = ProcessVectorContext::<T, C>::new(context);
                    process_vector_tile(
                        &data,
                        VectorTileRequest {
                            coords,
                            layers: fill_layers,
                        },
                        &mut pipeline_context,
                    )
                    .map_err(|e| ProcedureError::Execution(Box::new(e)))?;
                }
                Err(e) => {
                    log::error!("{e:?}");
                    for to_load in &fill_layers {
                        context
                            .send(<T as VectorTransferables>::LayerMissing::build_from(
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
