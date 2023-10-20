use std::{any::TypeId, collections::HashSet};

pub mod resources;
pub mod system;
pub mod tiles;
pub mod world;

#[derive(Default)]
pub struct GlobalQueryState {
    mutably_borrowed: HashSet<TypeId>,
}

pub trait QueryState<'s> {
    fn create(state: &'s mut GlobalQueryState) -> Self;
    fn clone_to<'a, S: QueryState<'a>>(&'a mut self) -> S;
}

pub struct EphemeralQueryState<'s> {
    state: &'s mut GlobalQueryState,
}

impl<'s> QueryState<'s> for EphemeralQueryState<'s> {
    fn create(state: &'s mut GlobalQueryState) -> Self {
        Self { state }
    }

    fn clone_to<'a, S: QueryState<'a>>(&'a mut self) -> S {
        S::create(self.state)
    }
}
