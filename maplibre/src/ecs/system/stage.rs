use log::info;

use crate::{
    context::MapContext,
    ecs::system::{function::IntoSystem, BoxedSystem, System},
    schedule::Stage,
};

pub struct SystemContainer {
    system: BoxedSystem,
}

#[derive(Default)]
pub struct SystemStage {
    systems: Vec<BoxedSystem>,
}

impl SystemStage {
    #[must_use]
    pub fn with_system(mut self, system: impl IntoSystem) -> Self {
        self.add_system(system);
        self
    }

    #[must_use]
    pub fn with_system_direct(mut self, system: impl System) -> Self {
        self.add_system_direct(system);
        self
    }

    pub fn add_system(&mut self, system: impl IntoSystem) -> &mut Self {
        self.systems.push(Box::new(system.into_system()));
        self
    }

    pub fn add_system_direct(&mut self, system: impl System) -> &mut Self {
        self.systems.push(Box::new(system));
        self
    }
}

impl Stage for SystemStage {
    fn run(&mut self, context: &mut MapContext) {
        for mut system in &mut self.systems {
            info!("system {}", system.name());
            system.run(context)
        }
    }
}
