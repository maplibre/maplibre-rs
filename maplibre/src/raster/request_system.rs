//! Requests tiles which are currently in view

use std::{borrow::Cow, collections::HashSet, marker::PhantomData, rc::Rc};

use crate::{
    context::MapContext,
    environment::{Environment, OffscreenKernelEnvironment},
    io::{
        apc::{AsyncProcedureCall, AsyncProcedureFuture, Context, Input, ProcedureError},
        source_type::{RasterSource, SourceType},
    },
    kernel::Kernel,
    raster::{
        process_raster::{process_raster_tile, ProcessRasterContext, RasterTileRequest},
        transferables::{LayerRasterMissing, RasterTransferables},
        RasterLayersDataComponent,
    },
    style::layer::LayerPaint,
    tcs::system::System,
};

pub struct RequestSystem<E: Environment, T: RasterTransferables> {
    kernel: Rc<Kernel<E>>,
    phantom_t: PhantomData<T>,
}

impl<E: Environment, T: RasterTransferables> RequestSystem<E, T> {
    pub fn new(kernel: &Rc<Kernel<E>>) -> Self {
        Self {
            kernel: kernel.clone(),
            phantom_t: Default::default(),
        }
    }
}

impl<E: Environment, T: RasterTransferables> System for RequestSystem<E, T> {
    fn name(&self) -> Cow<'static, str> {
        "raster_request".into()
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
                        .query::<&RasterLayersDataComponent>(coords)
                        .is_some()
                    {
                        continue;
                    }

                    world
                        .tiles
                        .spawn_mut(coords)
                        .unwrap()
                        .insert(RasterLayersDataComponent::default());

                    tracing::event!(tracing::Level::ERROR, %coords, "tile request started: {coords}");
                    log::info!("tile request started: {coords}");

                    self.kernel
                        .apc()
                        .call(
                            Input::TileRequest {
                                coords,
                                style: style.clone(), // TODO: Avoid cloning whole style
                            },
                            fetch_raster_apc::<
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
pub fn fetch_raster_apc<
    K: OffscreenKernelEnvironment,
    T: RasterTransferables,
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

        let raster_layers: HashSet<String> = style
            .layers
            .iter()
            .filter_map(|layer| {
                if matches!(layer.paint, Some(LayerPaint::Raster(_))) {
                    layer.source_layer.clone()
                } else {
                    None
                }
            })
            .collect();

        let client = kernel.source_client();

        if !raster_layers.is_empty() {
            let context = context.clone();
            let source = SourceType::Raster(RasterSource::default());

            match client.fetch(&coords, &source).await {
                Ok(data) => {
                    let data = data.into_boxed_slice();

                    let mut process_context = ProcessRasterContext::<T, C>::new(context);

                    process_raster_tile(&data, RasterTileRequest { coords }, &mut process_context)
                        .map_err(|e| ProcedureError::Execution(Box::new(e)))?;
                }
                Err(e) => {
                    log::error!("{e:?}");

                    context
                        .send(<T as RasterTransferables>::LayerRasterMissing::build_from(
                            coords,
                        ))
                        .map_err(ProcedureError::Send)?;
                }
            }
        }

        Ok(())
    })
}
