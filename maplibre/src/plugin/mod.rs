use std::rc::Rc;

use crate::{ecs::world::World, environment::Environment, kernel::Kernel, schedule::Schedule};

pub trait Plugin<E: Environment> {
    fn build(&self, schedule: &mut Schedule, kernel: Rc<Kernel<E>>, world: &mut World);
}
