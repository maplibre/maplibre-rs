use crate::{ecs::world::World, environment::Environment, kernel::Kernel};

pub trait Plugin<E: Environment> {
    fn build(&self, kernel: &mut Kernel<E>, world: &mut World);
}
