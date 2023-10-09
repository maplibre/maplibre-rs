use std::{borrow::Cow, marker::PhantomData, rc::Rc};

use crate::{
    context::MapContext,
    environment::Environment,
    io::apc::{AsyncProcedureCall, Message},
    kernel::Kernel,
    tcs::system::System,
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

impl<E: Environment, T: VectorTransferables> System for PopulateWorldSystem<E, T> {
    fn name(&self) -> Cow<'static, str> {
        "populate_world_system".into()
    }

    fn run(&mut self, MapContext { world, .. }: &mut MapContext) {
        for message in self.kernel.apc().receive(|message| {
            message.has_tag(T::TileTessellated::message_tag())
                || message.has_tag(T::LayerMissing::message_tag())
                || message.has_tag(T::LayerTessellated::message_tag())
                || message.has_tag(T::LayerIndexed::message_tag())
        }) {
            let message: Message = message;
            if message.has_tag(T::TileTessellated::message_tag()) {
                let message = message.into_transferable::<T::TileTessellated>();
                let Some(component) = world
                    .tiles
                    .query_mut::<&mut VectorLayersDataComponent>(message.coords())
                else {
                    continue;
                };

                component.done = true;
            } else if message.has_tag(T::LayerMissing::message_tag()) {
                let message = message.into_transferable::<T::LayerMissing>();
                let Some(component) = world
                    .tiles
                    .query_mut::<&mut VectorLayersDataComponent>(message.coords())
                else {
                    continue;
                };

                component
                    .layers
                    .push(VectorLayerData::Missing(message.to_layer()));
            } else if message.has_tag(T::LayerTessellated::message_tag()) {
                let message = message.into_transferable::<T::LayerTessellated>();
                // FIXME: Handle points!
                /*if message.is_empty() {
                    continue;
                }*/

                let Some(component) = world
                    .tiles
                    .query_mut::<&mut VectorLayersDataComponent>(message.coords())
                else {
                    continue;
                };

                component
                    .layers
                    .push(VectorLayerData::Available(message.to_layer()));
            } else if message.has_tag(T::LayerIndexed::message_tag()) {
                let message = message.into_transferable::<T::LayerIndexed>();
                world
                    .tiles
                    .geometry_index
                    .index_tile(&message.coords(), message.to_tile_index());
            }
        }
    }
}
