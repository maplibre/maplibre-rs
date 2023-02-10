use std::{borrow::Cow, marker::PhantomData, rc::Rc};

use crate::{
    context::MapContext,
    ecs::system::System,
    environment::Environment,
    io::apc::AsyncProcedureCall,
    kernel::Kernel,
    match_downcast,
    raster::{
        transferables::{LayerRaster, Transferables},
        RasterLayersDataComponent,
    },
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

impl<E: Environment, T: Transferables> System for PopulateWorldSystem<E, T> {
    fn name(&self) -> Cow<'static, str> {
        "populate_world_system".into()
    }

    fn run(
        &mut self,
        MapContext {
            world, renderer, ..
        }: &mut MapContext,
    ) {
        for message in self.kernel.apc().receive(|message| {
            let transferable = &message.transferable;
            transferable.is::<<T as Transferables>::LayerRaster>()
        }) {
            match_downcast!(message.transferable, {
                message: <T as Transferables>::LayerRaster => {
                    let Some(component) = world
                        .tiles
                        .query_mut::<&mut RasterLayersDataComponent>(message.coords()) else { continue; };

                    component.layers.push(message.to_layer());
                },
                  // FIXME tcs: Add RasterLayerUnvailable
                _ => {}
            });
        }
    }
}
