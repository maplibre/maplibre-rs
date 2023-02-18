use std::rc::Rc;

use crate::{
    environment::Environment, kernel::Kernel, render::graph::RenderGraph, schedule::Schedule,
    tcs::world::World,
};

pub trait Plugin<E: Environment> {
    fn build(
        &self,
        schedule: &mut Schedule,
        kernel: Rc<Kernel<E>>,
        world: &mut World,
        graph: &mut RenderGraph,
    );
}
