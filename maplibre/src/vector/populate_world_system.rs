use std::{borrow::Cow, rc::Rc};

use crate::{
    context::MapContext,
    ecs::system::System,
    environment::Environment,
    io::{
        apc::AsyncProcedureCall,
        transferables::{
            LayerIndexed, LayerTessellated, LayerUnavailable, TileTessellated, Transferables,
        },
    },
    kernel::Kernel,
    match_downcast,
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
        let geometry_index = &mut world.geometry_index;

        type Type<E: Environment> =
            <E::AsyncProcedureCall as AsyncProcedureCall<E::HttpClient>>::Transferables;

        for message in self.kernel.apc().receive(|message| {
            let transferable = &message.transferable;
            transferable.is::<<Type<E> as Transferables>::TileTessellated>()
                || transferable.is::<<Type<E> as Transferables>::LayerUnavailable>()
                || transferable.is::<<Type<E> as Transferables>::LayerTessellated>()
                || transferable.is::<<Type<E> as Transferables>::LayerIndexed>()
        }) {
            match_downcast!(message.transferable, {
                message: <Type<E> as Transferables>::TileTessellated => {
                    let Some(component) = world
                        .tiles
                        .query_mut::<&mut VectorLayersDataComponent>(message.coords()) else { continue; };

                    component.done = true;
                },
                message: <Type<E> as Transferables>::LayerUnavailable => {
                    let Some(component) = world
                        .tiles
                        .query_mut::<&mut VectorLayersDataComponent>(message.coords()) else { continue; };

                    component.done = true;
                },
                message: <Type<E> as Transferables>::LayerTessellated => {
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
                message: <Type<E> as Transferables>::LayerIndexed => {
                     geometry_index.index_tile(&message.coords(), message.to_tile_index());
                },
                _ => {}
            });
        }
    }
}
