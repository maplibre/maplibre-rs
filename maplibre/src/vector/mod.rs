mod populate_world_system;
mod queue_system;
mod render_commands;
mod resource_system;
mod tile_view_pattern_system;
mod upload_system;

use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};

use crate::{
    coords::WorldTileCoords,
    ecs::{component::TileComponent, system::SystemContainer, world::World},
    environment::Environment,
    kernel::Kernel,
    plugin::Plugin,
    render::{
        eventually::Eventually,
        render_phase::{LayerItem, RenderPhase, TileMaskItem},
        resource::{BufferPool, IndexEntry},
        shaders::{ShaderFeatureStyle, ShaderLayerMetadata},
        stages::RenderStageLabel,
        tile_view_pattern::{TileShape, TileViewPattern},
        ShaderVertex,
    },
    schedule::Schedule,
    systems::request_system::RequestSystem,
    tessellation::{IndexDataType, OverAlignedVertexBuffer},
    vector::{
        populate_world_system::PopulateWorldSystem, queue_system::queue_system,
        resource_system::resource_system, tile_view_pattern_system::tile_view_pattern_system,
        upload_system::upload_system,
    },
};

pub struct VectorPipeline(wgpu::RenderPipeline);
impl Deref for VectorPipeline {
    type Target = wgpu::RenderPipeline;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct MaskPipeline(wgpu::RenderPipeline);
impl Deref for MaskPipeline {
    type Target = wgpu::RenderPipeline;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct DebugPipeline(wgpu::RenderPipeline);
impl Deref for DebugPipeline {
    type Target = wgpu::RenderPipeline;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type VectorBufferPool = BufferPool<
    wgpu::Queue,
    wgpu::Buffer,
    ShaderVertex,
    IndexDataType,
    ShaderLayerMetadata,
    ShaderFeatureStyle,
>;

pub type WgpuTileViewPattern = TileViewPattern<wgpu::Queue, wgpu::Buffer>;

pub struct VectorPlugin;

impl<E: Environment> Plugin<E> for VectorPlugin {
    fn build(&self, schedule: &mut Schedule, kernel: Rc<Kernel<E>>, world: &mut World) {
        // FIXME tcs: Move to rendering core
        let resources = &mut world.resources;
        resources.init::<RenderPhase<LayerItem>>();
        resources.init::<RenderPhase<TileMaskItem>>();

        // buffer_pool
        resources.insert(Eventually::<VectorBufferPool>::Uninitialized);

        // tile_view_pattern:
        // FIXME tcs: Move to rendering core
        resources.insert(Eventually::<WgpuTileViewPattern>::Uninitialized);

        // vector_tile_pipeline
        resources.insert(Eventually::<VectorPipeline>::Uninitialized);
        // mask_pipeline
        // FIXME tcs: Move to rendering core?
        resources.insert(Eventually::<MaskPipeline>::Uninitialized);
        // debug_pipeline
        resources.insert(Eventually::<DebugPipeline>::Uninitialized);

        // FIXME tcs: Move to rendering core
        resources.insert(RenderPhase::<LayerItem>::default());

        // FIXME tcs: Move to rendering core
        schedule.add_system_to_stage(
            &RenderStageLabel::Extract,
            SystemContainer::new(RequestSystem::new(&kernel)),
        );
        schedule.add_system_to_stage(
            &RenderStageLabel::Extract,
            SystemContainer::new(PopulateWorldSystem::new(&kernel)),
        );
        schedule.add_system_to_stage(&RenderStageLabel::Prepare, resource_system);
        schedule.add_system_to_stage(&RenderStageLabel::Prepare, tile_view_pattern_system);
        schedule.add_system_to_stage(&RenderStageLabel::Queue, upload_system); // FIXME tcs: Upload updates the TileView in tileviewpattern -> upload most run before prepare
        schedule.add_system_to_stage(&RenderStageLabel::Queue, queue_system);
    }
}

pub struct AvailableVectorLayerData {
    pub coords: WorldTileCoords,
    pub source_layer: String,
    pub buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
    /// Holds for each feature the count of indices.
    pub feature_indices: Vec<u32>,
}

pub struct UnavailableVectorLayerData {
    pub coords: WorldTileCoords,
    pub source_layer: String,
}

pub enum VectorLayerData {
    Available(AvailableVectorLayerData),
    Unavailable(UnavailableVectorLayerData),
}

#[derive(Default)]
pub struct VectorLayersDataComponent {
    pub done: bool,
    pub layers: Vec<VectorLayerData>,
}

impl TileComponent for VectorLayersDataComponent {}

#[derive(Default)]
pub struct VectorLayersIndicesComponent {
    pub layers: Vec<IndexEntry>,
}

impl TileComponent for VectorLayersIndicesComponent {}
