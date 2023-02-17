use std::borrow::Cow;

use crate::{context::MapContext, tcs::system::function::IntoSystem};

mod function;
pub mod stage;

/// An system that can be added to a [`Schedule`](crate::schedule::Schedule)
pub trait System: 'static {
    /// Returns the system's name.
    fn name(&self) -> Cow<'static, str>;

    fn run(&mut self, context: &mut MapContext);
}

/// A convenience type alias for a boxed [`System`] trait object.
pub type BoxedSystem = Box<dyn System>;

pub struct SystemContainer {
    system: BoxedSystem,
}

impl SystemContainer {
    pub fn new<S: System>(system: S) -> Self {
        Self {
            system: Box::new(system),
        }
    }
}

pub trait IntoSystemContainer {
    fn into_container(self) -> SystemContainer;
}

impl<S> IntoSystemContainer for S
where
    S: IntoSystem,
{
    fn into_container(self) -> SystemContainer {
        SystemContainer {
            system: Box::new(IntoSystem::into_system(self)),
        }
    }
}

impl IntoSystemContainer for SystemContainer {
    fn into_container(self) -> SystemContainer {
        self
    }
}
