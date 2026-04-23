use crate::{environment::Environment, plugin::Plugin};

pub mod queue_system;
pub mod render_commands;
pub mod resource_system;

pub struct BackgroundPlugin;

impl Default for BackgroundPlugin {
    fn default() -> Self {
        Self
    }
}

impl<E: Environment> Plugin<E> for BackgroundPlugin {
    fn build(
        &self,
        schedule: &mut crate::schedule::Schedule,
        _kernel: std::rc::Rc<crate::kernel::Kernel<E>>,
        world: &mut crate::tcs::world::World,
        _graph: &mut crate::render::graph::RenderGraph,
    ) {
        world.resources.insert(
            crate::render::eventually::Eventually::<
                crate::background::resource_system::BackgroundRenderPipeline,
            >::Uninitialized,
        );

        schedule.add_system_to_stage(
            crate::render::RenderStageLabel::Queue,
            queue_system::queue_system,
        );
        schedule.add_system_to_stage(
            crate::render::RenderStageLabel::Prepare,
            crate::background::resource_system::resource_system,
        );
    }
}
