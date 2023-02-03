use std::borrow::Cow;

use crate::ecs::world::World;

/// An ECS system that can be added to a [`Schedule`](crate::schedule::Schedule)
///
/// Systems are functions with all arguments implementing
/// [`SystemParam`](crate::system::SystemParam).
///
/// Systems are added to an application using `App::add_system(my_system)`
/// or similar methods, and will generally run once per pass of the main loop.
///
/// Systems are executed in parallel, in opportunistic order; data access is managed automatically.
/// It's possible to specify explicit execution order between specific systems,
/// see [`SystemDescriptor`](crate::schedule::SystemDescriptor).
pub trait System: Send + Sync + 'static {
    /// The system's input. See [`In`](crate::system::In) for
    /// [`FunctionSystem`](crate::system::FunctionSystem)s.
    type In;
    /// The system's output.
    type Out;
    /// Returns the system's name.
    fn name(&self) -> Cow<'static, str>;

    fn run(&mut self, input: Self::In, world: &mut World) -> Self::Out {
        unimplemented!()
    }
}

/// A convenience type alias for a boxed [`System`] trait object.
pub type BoxedSystem<In = (), Out = ()> = Box<dyn System<In = In, Out = Out>>;
