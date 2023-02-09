use std::{borrow::Cow, rc::Rc};

use crate::{
    context::MapContext,
    ecs::system::System,
    environment::Environment,
    io::{
        apc::{AsyncProcedureCall, Message},
        transferables::{LayerRaster, LayerTessellated, LayerUnavailable},
    },
    kernel::Kernel,
    raster::RasterLayersDataComponent,
    vector::{VectorLayerData, VectorLayersDataComponent},
};

pub struct PopulateWorldSystem<E: Environment> {
    kernel: Rc<Kernel<E>>,
}

impl<E: Environment> PopulateWorldSystem<E> {
    pub fn new(kernel: &Rc<Kernel<E>>) -> Self {
        Self {
            kernel: kernel.clone(),
        }
    }
}

impl<E: Environment> System for PopulateWorldSystem<E> {
    fn name(&self) -> Cow<'static, str> {
        "populate_world_system".into()
    }

    fn run(
        &mut self,
        MapContext {
            world, renderer, ..
        }: &mut MapContext,
    ) {
        for result in self.kernel.apc().receive(|message| {
            matches!(
                message,
                Message::LayerRaster(_) // FIXME tcs: Add RasterLayerUnavailable
            )
        }) {
            match result {
                Message::LayerRaster(message) => {
                    let Some(component) = world
                        .tiles
                        .query_mut::<&mut RasterLayersDataComponent>(message.coords()) else { continue; };

                    component.layers.push(message.to_layer());
                }
                // FIXME tcs: Add RasterLayerUnvailable
                _ => {}
            };
        }
    }
}
