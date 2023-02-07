use std::{any::TypeId, collections::HashMap};

use crate::render::render_phase::{Draw, PhaseItem};

/// /// A [`Draw`] function identifier.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct DrawFunctionId(usize);

/// Stores all draw functions for the [`PhaseItem`] type.
/// For retrieval they are associated with their [`TypeId`].
pub struct DrawFunctions<P: PhaseItem> {
    pub draw_functions: Vec<Box<dyn Draw<P>>>,
    pub indices: HashMap<TypeId, DrawFunctionId>,
}

impl<P: PhaseItem> Default for DrawFunctions<P> {
    fn default() -> Self {
        Self {
            draw_functions: vec![],
            indices: Default::default(),
        }
    }
}

impl<P: PhaseItem> DrawFunctions<P> {
    /// Adds the [`Draw`] function and associates it to its own type.
    pub fn add<T: Draw<P>>(&mut self, draw_function: T) -> DrawFunctionId {
        self.add_with::<T, T>(draw_function)
    }

    /// Adds the [`Draw`] function and associates it to the type `T`
    pub fn add_with<T: 'static, D: Draw<P>>(&mut self, draw_function: D) -> DrawFunctionId {
        self.draw_functions.push(Box::new(draw_function));
        let id = DrawFunctionId(self.draw_functions.len() - 1);
        self.indices.insert(TypeId::of::<T>(), id);
        id
    }

    /// Retrieves the [`Draw`] function corresponding to the `id` mutably.
    pub fn get_mut(&mut self, id: DrawFunctionId) -> Option<&mut dyn Draw<P>> {
        self.draw_functions.get_mut(id.0).map(|f| &mut **f)
    }

    /// Retrieves the [`Draw`] function corresponding to the `id`.
    pub fn get(&self, id: DrawFunctionId) -> Option<&dyn Draw<P>> {
        self.draw_functions.get(id.0).map(|f| &**f)
    }

    /// Retrieves the id of the [`Draw`] function corresponding to their associated type `T`.
    pub fn get_id<T: 'static>(&self) -> Option<DrawFunctionId> {
        self.indices.get(&TypeId::of::<T>()).copied()
    }
}
