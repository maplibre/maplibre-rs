use std::{borrow::Cow, rc::Rc};

use crate::{
    context::MapContext,
    ecs::system::System,
    environment::Environment,
    io::{
        apc::{AsyncProcedureCall, Message},
        transferables::{
            LayerIndexed, LayerRaster, LayerTessellated, LayerUnavailable, TileTessellated,
        },
    },
    kernel::Kernel,
    vector::{UnavailableVectorLayerData, VectorLayerData, VectorLayersDataComponent},
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
        let tiles = &mut world.tiles;
        let geometry_index = &mut world.geometry_index;

        for result in self.kernel.apc().receive(|message| {
            matches!(
                message,
                Message::TileTessellated(_)
                    | Message::LayerTessellated(_)
                    | Message::LayerIndexed(_)
                    | Message::LayerUnavailable(_)
            )
        }) {
            match result {
                Message::TileTessellated(message) => {
                    let Some(component) = world
                        .tiles
                        .query_mut::<&mut VectorLayersDataComponent>(message.coords()) else { continue; };

                    component.done = true;
                }
                Message::LayerUnavailable(message) => {
                    let Some(component) = world
                        .tiles
                        .query_mut::<&mut VectorLayersDataComponent>(message.coords()) else { continue; };

                    component
                        .layers
                        .push(VectorLayerData::Unavailable(message.to_layer()));
                }
                Message::LayerTessellated(message) => {
                    // FIXME: Handle points!
                    /*if message.is_empty() {
                        continue;
                    }*/

                    let Some(component) = world
                        .tiles
                        .query_mut::<&mut VectorLayersDataComponent>(message.coords()) else { continue; };

                    component
                        .layers
                        .push(VectorLayerData::Available(message.to_layer()));
                }
                Message::LayerIndexed(message) => {
                    let coords = message.coords();
                    geometry_index.index_tile(&coords, message.to_tile_index());
                }
                _ => {}
            }
        }
    }
}
