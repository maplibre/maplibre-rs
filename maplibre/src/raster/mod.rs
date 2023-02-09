mod populate_world_system;
mod queue_system;
mod render_commands;
mod resource;
mod resource_system;
mod upload_system;

use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};

use image::RgbaImage;
pub use resource::RasterResources;

use crate::{
    coords::WorldTileCoords,
    ecs::{system::SystemContainer, tiles::TileComponent, world::World},
    environment::Environment,
    kernel::Kernel,
    plugin::Plugin,
    raster::{
        populate_world_system::PopulateWorldSystem, queue_system::queue_system,
        resource_system::resource_system, upload_system::upload_system,
    },
    render::{eventually::Eventually, stages::RenderStageLabel},
    schedule::Schedule,
}; // FIXME tcs: avoid making this public

pub struct RasterPlugin;

impl<E: Environment> Plugin<E> for RasterPlugin {
    fn build(&self, schedule: &mut Schedule, kernel: Rc<Kernel<E>>, world: &mut World) {
        // raster_resources
        world
            .resources
            .insert(Eventually::<RasterResources>::Uninitialized);

        schedule.add_system_to_stage(
            &RenderStageLabel::Extract,
            SystemContainer::new(PopulateWorldSystem::new(&kernel)),
        );

        schedule.add_system_to_stage(&RenderStageLabel::Prepare, resource_system);

        schedule.add_system_to_stage(&RenderStageLabel::Queue, upload_system);
        schedule.add_system_to_stage(&RenderStageLabel::Queue, queue_system); // FIXME tcs: Upload updates the TileView in tileviewpattern -> upload most run before prepare
    }
}

pub struct RasterLayerData {
    pub coords: WorldTileCoords,
    pub source_layer: String,
    pub image: RgbaImage,
}

// FIXME tcs: Add AvailableRasterLayerData and UnavailableRasterLayerData

#[derive(Default)]
pub struct RasterLayersDataComponent {
    pub layers: Vec<RasterLayerData>,
}

impl TileComponent for RasterLayersDataComponent {}
