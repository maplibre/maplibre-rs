use std::{borrow::Cow, marker::PhantomData, rc::Rc};

use crate::{
    context::MapContext,
    ecs::system::System,
    environment::Environment,
    io::apc::AsyncProcedureCall,
    kernel::Kernel,
    match_downcast,
    vector::{transferables::*, VectorLayerData, VectorLayersDataComponent},
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
        let geometry_index = &mut world.geometry_index;

        for message in self.kernel.apc().receive(|message| {
            let transferable = &message.transferable;
            transferable.is::<<T as Transferables>::TileTessellated>()
                || transferable.is::<<T as Transferables>::LayerUnavailable>()
                || transferable.is::<<T as Transferables>::LayerTessellated>()
                || transferable.is::<<T as Transferables>::LayerIndexed>()
        }) {
            match_downcast!(message.transferable, {
                message: <T as Transferables>::TileTessellated => {
                    let Some(component) = world
                        .tiles
                        .query_mut::<&mut VectorLayersDataComponent>(message.coords()) else { continue; };

                    component.done = true;
                },
                message: <T as Transferables>::LayerUnavailable => {
                    let Some(component) = world
                        .tiles
                        .query_mut::<&mut VectorLayersDataComponent>(message.coords()) else { continue; };

                    component.done = true;
                },
                message: <T as Transferables>::LayerTessellated => {
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
                },
                message: <T as Transferables>::LayerIndexed => {
                     geometry_index.index_tile(&message.coords(), message.to_tile_index());
                },
                _ => {}
            });
        }
    }
}
