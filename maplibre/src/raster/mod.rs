use std::{marker::PhantomData, rc::Rc};

use image::RgbaImage;

use crate::{
    coords::WorldTileCoords,
    environment::Environment,
    kernel::Kernel,
    plugin::Plugin,
    raster::{
        populate_world_system::PopulateWorldSystem, queue_system::queue_system,
        request_system::RequestSystem, resource::RasterResources, resource_system::resource_system,
        upload_system::upload_system,
    },
    render::{eventually::Eventually, tile_view_pattern::ViewTileSources, RenderStageLabel},
    schedule::Schedule,
    tcs::{system::SystemContainer, tiles::TileComponent, world::World},
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

pub use transferables::{
    DefaultRasterTransferables, LayerRaster, LayerRasterMissing, RasterTransferables,
};

use crate::render::graph::RenderGraph;

pub struct RasterPlugin<T>(PhantomData<T>);

impl<T: RasterTransferables> Default for RasterPlugin<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<E: Environment, T: RasterTransferables> Plugin<E> for RasterPlugin<T> {
    fn build(
        &self,
        schedule: &mut Schedule,
        kernel: Rc<Kernel<E>>,
        world: &mut World,
        _graph: &mut RenderGraph,
    ) {
        world
            .resources
            .insert(Eventually::<RasterResources>::Uninitialized);

        world
            .resources
            .get_or_init_mut::<ViewTileSources>()
            .add_resource_query::<&Eventually<RasterResources>>();

        schedule.add_system_to_stage(
            RenderStageLabel::Extract,
            SystemContainer::new(RequestSystem::<E, T>::new(&kernel)),
        );
        schedule.add_system_to_stage(
            RenderStageLabel::Extract,
            SystemContainer::new(PopulateWorldSystem::<E, T>::new(&kernel)),
        );
        schedule.add_system_to_stage(RenderStageLabel::Prepare, resource_system);
        schedule.add_system_to_stage(RenderStageLabel::Queue, upload_system);
        schedule.add_system_to_stage(RenderStageLabel::Queue, queue_system); // FIXME tcs: Upload updates the TileView in tileviewpattern -> upload most run before prepare
    }
}

pub struct AvailableRasterLayerData {
    pub coords: WorldTileCoords,
    pub source_layer: String,
    pub image: RgbaImage,
}

pub struct MissingRasterLayerData {
    pub coords: WorldTileCoords,
    pub source_layer: String,
}

pub enum RasterLayerData {
    Available(AvailableRasterLayerData),
    Missing(MissingRasterLayerData),
}

#[derive(Default)]
pub struct RasterLayersDataComponent {
    pub layers: Vec<RasterLayerData>,
}

impl TileComponent for RasterLayersDataComponent {}
