use log::info;

use crate::{
    context::MapContext,
    ecs::system::{
        function::IntoSystem, BoxedSystem, IntoSystemContainer, System, SystemContainer,
    },
    schedule::Stage,
};

#[derive(Default)]
pub struct SystemStage {
    systems: Vec<SystemContainer>,
}

impl SystemStage {
    #[must_use]
    pub fn with_system(mut self, system: impl IntoSystemContainer) -> Self {
        self.add_system(system);
        self
    }

    pub fn add_system(&mut self, system: impl IntoSystemContainer) -> &mut Self {
        self.systems.push(system.into_container());
        self
    }
}

impl Stage for SystemStage {
    fn run(&mut self, context: &mut MapContext) {
        for mut container in &mut self.systems {
            #[cfg(feature = "trace")]
            let _span =
                tracing::info_span!("system", name = container.system.name().as_ref()).entered();
            container.system.run(context)
        }
    }
}
