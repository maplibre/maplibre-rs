use std::{marker::PhantomData, rc::Rc};

use image::RgbaImage;
pub use resource::RasterResources; // FIXME tcs
pub use transferables::*;

use crate::{
    coords::WorldTileCoords,
    ecs::{system::SystemContainer, tiles::TileComponent, world::World},
    environment::Environment,
    kernel::Kernel,
    plugin::Plugin,
    raster::{
        populate_world_system::PopulateWorldSystem, queue_system::queue_system,
        request_system::RequestSystem, resource_system::resource_system,
        upload_system::upload_system,
    },
    render::{eventually::Eventually, stages::RenderStageLabel},
    schedule::Schedule,
};

mod populate_world_system;
mod process_raster;
mod queue_system;
mod render_commands;
mod request_system;
mod resource;
mod resource_system;
mod transferables;
mod upload_system;

// FIXME tcs: avoid making this public

pub struct RasterPlugin<T>(PhantomData<T>);

impl<T: RasterTransferables> Default for RasterPlugin<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<E: Environment, T: RasterTransferables> Plugin<E> for RasterPlugin<T> {
    fn build(&self, schedule: &mut Schedule, kernel: Rc<Kernel<E>>, world: &mut World) {
        // raster_resources
        world
            .resources
            .insert(Eventually::<RasterResources>::Uninitialized);

        schedule.add_system_to_stage(
            &RenderStageLabel::Extract,
            SystemContainer::new(RequestSystem::<E, T>::new(&kernel)),
        );
        schedule.add_system_to_stage(
            &RenderStageLabel::Extract,
            SystemContainer::new(PopulateWorldSystem::<E, T>::new(&kernel)),
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
