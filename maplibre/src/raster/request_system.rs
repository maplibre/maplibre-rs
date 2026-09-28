//! Requests tiles which are currently in view

use std::{borrow::Cow, collections::HashSet, marker::PhantomData, rc::Rc};

use crate::{
    context::MapContext,
    environment::{Environment, OffscreenKernel},
    io::{
        apc::{AsyncProcedureCall, AsyncProcedureFuture, Context, Input, ProcedureError},
        tile_sources::{clamp_to_max_zoom, source_layer_groups, source_max_zoom, TileKind},
    },
    kernel::Kernel,
    raster::{
        process_raster::{process_raster_tile, ProcessRasterContext, RasterTileRequest},
        transferables::{LayerRasterMissing, RasterTransferables},
        RasterLayersDataComponent,
    },
    render::{
        projection::view_region_for_projection, tile_view_pattern::DEFAULT_TILE_SIZE,
        view_state::ViewStatePadding,
    },
    tcs::system::{System, SystemResult},
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
    ) -> SystemResult {
        let view_region = view_region_for_projection(
            style,
            view_state,
            view_state.zoom().zoom_level(DEFAULT_TILE_SIZE),
            ViewStatePadding::Loose,
        )
        .map_err(|error| {
            tracing::error!(%error, "unable to select raster request tiles");
            crate::tcs::system::SystemError::Setup
        })?;

        if view_state.did_camera_change() || view_state.did_zoom_change() {
            if let Some(view_region) = &view_region {
                let max_zoom = source_max_zoom(style, TileKind::Raster);
                let mut requested = HashSet::new();

                for coords in view_region.iter() {
                    // Above the source maximum zoom the ancestor tile is fetched once and the
                    // view pattern scales it into every descendant in view.
                    let coords = clamp_to_max_zoom(coords, max_zoom);
                    if coords.build_quad_key().is_none() || !requested.insert(coords) {
                        continue;
                    }

                    // TODO: Make tessellation depend on style? So maybe we need to request even if it exists
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
                        .expect("unable to spawn a raster tile")
                        .insert(RasterLayersDataComponent::default());

                    tracing::debug!(%coords, "tile request started");

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
                        .expect("unable to call APC"); // TODO: Remove unwrap
                }
            }
        }

        view_state.update_references();

        Ok(())
    }
}
pub fn fetch_raster_apc<K: OffscreenKernel, T: RasterTransferables, C: Context + Clone + Send>(
    input: Input,
    context: C,
    kernel: K,
) -> AsyncProcedureFuture {
    Box::pin(async move {
        let Input::TileRequest { coords, style } = input else {
            return Err(ProcedureError::IncompatibleInput);
        };

        let client = kernel.source_client();

        for group in source_layer_groups(&style, TileKind::Raster) {
            let context = context.clone();
            match client.fetch(&coords, &group.source).await {
                Ok(data) => {
                    let data = data.into_boxed_slice();

                    let mut process_context = ProcessRasterContext::<T, C>::new(context);

                    process_raster_tile(&data, RasterTileRequest { coords }, &mut process_context)
                        .map_err(|e| ProcedureError::Execution(Box::new(e)))?;
                }
                Err(error) => {
                    tracing::error!(
                        %coords,
                        source = ?group.source_name,
                        %error,
                        "raster tile fetch failed"
                    );

                    context
                        .send_back(<T as RasterTransferables>::LayerRasterMissing::build_from(
                            coords,
                        ))
                        .map_err(ProcedureError::Send)?;
                }
            }
        }

        Ok(())
    })
}
