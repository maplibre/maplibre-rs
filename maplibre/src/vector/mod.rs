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
        populate_world_system::PopulateWorldSystem,
        queue_system::queue_system,
        render_commands::{DrawMasks, DrawVectorTiles},
        resource_system::resource_system,
        tile_view_pattern_system::tile_view_pattern_system,
        upload_system::upload_system,
    },
};

// FIXME: Simplify those NewTypes

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

pub struct VectorPlugin;

impl<E: Environment> Plugin<E> for VectorPlugin {
    fn build(&self, schedule: &mut Schedule, kernel: Rc<Kernel<E>>, world: &mut World) {
        // FIXME: Split into several plugins

        world.init_resource::<RenderPhase<LayerItem>>();
        world.init_resource::<RenderPhase<TileMaskItem>>();

        // buffer_pool
        world.insert_resource(Eventually::<VectorBufferPool>::Uninitialized);

        // tile_view_pattern:
        world.insert_resource(
            // FIXME: Simplify type
            Eventually::<TileViewPattern<wgpu::Queue, wgpu::Buffer>>::Uninitialized,
        );

        // vector_tile_pipeline
        world.insert_resource(Eventually::<VectorPipeline>::Uninitialized);
        // mask_pipeline
        world.insert_resource(Eventually::<MaskPipeline>::Uninitialized);
        // debug_pipeline
        world.insert_resource(Eventually::<DebugPipeline>::Uninitialized);

        // TODO: move
        world.insert_resource(RenderPhase::<LayerItem>::default());

        // TODO: move
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
        schedule.add_system_to_stage(&RenderStageLabel::Queue, upload_system); // TODO Upload updates the TileView in tileviewpattern -> upload most run before prepare
        schedule.add_system_to_stage(&RenderStageLabel::Queue, queue_system);
    }
}

pub struct VectorLayerComponent {
    coords: WorldTileCoords,
    layer_name: String,
    buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
    /// Holds for each feature the count of indices.
    feature_indices: Vec<u32>,
}

impl TileComponent for VectorLayerComponent {}

#[derive(Debug)]
pub struct VectorLayersComponent {
    pub entries: Vec<IndexEntry>,
}

impl TileComponent for VectorLayersComponent {}
