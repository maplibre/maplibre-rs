use crate::{
    context::MapContext,
    ecs::system::{function::IntoSystem, BoxedSystem},
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

    pub fn add_system(&mut self, system: impl IntoSystem) -> &mut Self {
        self.systems.push(Box::new(system.into_system()));
        self
    }
}

impl Stage for SystemStage {
    fn run(&mut self, context: &mut MapContext) {
        for mut system in &mut self.systems {
            system.run(context)
        }
    }
}
