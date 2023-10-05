use std::{borrow::Cow, marker::PhantomData, rc::Rc};

use crate::{
    context::MapContext,
    environment::Environment,
    io::apc::{AsyncProcedureCall, Message},
    kernel::Kernel,
    raster::{
        transferables::{LayerRaster, LayerRasterMissing, RasterTransferables},
        RasterLayerData, RasterLayersDataComponent,
    },
    tcs::system::System,
};

pub struct PopulateWorldSystem<E: Environment, T> {
    kernel: Rc<Kernel<E>>,
    phantom_t: PhantomData<T>,
}

impl<E: Environment, T> PopulateWorldSystem<E, T> {
    pub fn new(kernel: &Rc<Kernel<E>>) -> Self {
        Self {
            kernel: kernel.clone(),
            phantom_t: Default::default(),
        }
    }
}

impl<E: Environment, T: RasterTransferables> System for PopulateWorldSystem<E, T> {
    fn name(&self) -> Cow<'static, str> {
        "populate_world_system".into()
    }

    fn run(&mut self, MapContext { world, .. }: &mut MapContext) {
        for message in self.kernel.apc().receive(|message| {
            message.has_tag(T::LayerRaster::message_tag())
                || message.has_tag(T::LayerRasterMissing::message_tag())
        }) {
            let message: Message = message;
            if message.has_tag(T::LayerRaster::message_tag()) {
                let message = message.into_transferable::<T::LayerRaster>();
                let Some(component) = world
                    .tiles
                    .query_mut::<&mut RasterLayersDataComponent>(message.coords())
                else {
                    continue;
                };

                component
                    .layers
                    .push(RasterLayerData::Available(message.to_layer()));
            } else if message.has_tag(T::LayerRaster::message_tag()) {
                let message = message.into_transferable::<T::LayerRasterMissing>();
                let Some(component) = world
                    .tiles
                    .query_mut::<&mut RasterLayersDataComponent>(message.coords())
                else {
                    continue;
                };

                component
                    .layers
                    .push(RasterLayerData::Missing(message.to_layer()));
            }
        }
    }
}
