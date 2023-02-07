mod populate_world_system;
mod queue_system;
mod render_commands;
mod resource_system;
mod upload_system;

use std::{
    alloc::System,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use crate::{
    ecs::{
        system::{stage::SystemStage, SystemContainer},
        world::World,
    },
    environment::Environment,
    kernel::Kernel,
    plugin::Plugin,
    raster::{
        populate_world_system::PopulateWorldSystem, queue_system::queue_system,
        render_commands::DrawRasterTiles, resource_system::resource_system,
        upload_system::upload_system,
    },
    render::{
        eventually::Eventually,
        render_phase::{LayerItem, RenderPhase},
        resource::RasterResources,
        stages::RenderStageLabel,
        tile_view_pattern::TileShape,
    },
    schedule::Schedule,
};

pub struct RasterPlugin;

impl<E: Environment> Plugin<E> for RasterPlugin {
    fn build(&self, schedule: &mut Schedule, kernel: Rc<Kernel<E>>, world: &mut World) {
        // raster_resources
        world.insert_resource(Eventually::<RasterResources>::Uninitialized);

        schedule.add_system_to_stage(
            &RenderStageLabel::Extract,
            SystemContainer::new(PopulateWorldSystem::new(&kernel)),
        );

        schedule.add_system_to_stage(&RenderStageLabel::Prepare, resource_system);

        schedule.add_system_to_stage(&RenderStageLabel::Queue, upload_system);
        schedule.add_system_to_stage(&RenderStageLabel::Queue, queue_system); // TODO Upload updates the TileView in tileviewpattern -> upload most run before prepare
    }
}
