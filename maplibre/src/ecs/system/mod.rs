use std::borrow::Cow;

use crate::{context::MapContext, ecs::world::World, environment::Environment};

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
