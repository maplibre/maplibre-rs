use std::{marker::PhantomData, ops::Deref, rc::Rc};

use crate::{
    coords::WorldTileCoords,
    environment::Environment,
    kernel::Kernel,
    plugin::Plugin,
    render::{
        eventually::Eventually,
        shaders::{ShaderFeatureStyle, ShaderLayerMetadata},
        tile_view_pattern::{HasTile, ViewTileSources},
        RenderStageLabel, ShaderVertex,
    },
    schedule::Schedule,
    tcs::{system::SystemContainer, tiles::TileComponent, world::World},
    tessellation::{IndexDataType, OverAlignedVertexBuffer},
    vector::{
        populate_world_system::PopulateWorldSystem, queue_system::queue_system,
        request_system::RequestSystem, resource::BufferPool, resource_system::resource_system,
        upload_system::upload_system,
    },
};

mod populate_world_system;
mod process_vector;
mod queue_system;
mod render_commands;
mod request_system;
mod resource;
mod resource_system;
mod transferables;
mod upload_system;

pub use process_vector::*;
pub use transferables::{
    DefaultVectorTransferables, LayerIndexed, LayerMissing, LayerTessellated, TileTessellated,
    VectorTransferables,
};

use crate::render::graph::RenderGraph;

struct VectorPipeline(wgpu::RenderPipeline);
impl Deref for VectorPipeline {
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

pub struct VectorPlugin<T>(PhantomData<T>);

impl<T: VectorTransferables> Default for VectorPlugin<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

// FIXME: Is this the correct way to do this? Ideally we want to wait until all layers are uploaded to the gpu?
#[derive(Default)]
struct VectorTilesDone;

impl HasTile for VectorTilesDone {
    fn has_tile(&self, coords: WorldTileCoords, world: &World) -> bool {
        let Some(vector_layers_indices) = world.tiles.query::<&VectorLayersDataComponent>(coords)
        else {
            return false;
        };

        vector_layers_indices.done
    }
}

impl<E: Environment, T: VectorTransferables> Plugin<E> for VectorPlugin<T> {
    fn build(
        &self,
        schedule: &mut Schedule,
        kernel: Rc<Kernel<E>>,
        world: &mut World,
        _graph: &mut RenderGraph,
    ) {
        let resources = &mut world.resources;

        resources.insert(Eventually::<VectorBufferPool>::Uninitialized);
        resources.insert(Eventually::<VectorPipeline>::Uninitialized);

        resources
            .get_or_init_mut::<ViewTileSources>()
            .add_resource_query::<&Eventually<VectorBufferPool>>()
            .add::<VectorTilesDone>();

        schedule.add_system_to_stage(
            RenderStageLabel::Extract,
            SystemContainer::new(RequestSystem::<E, T>::new(&kernel)),
        );
        schedule.add_system_to_stage(
            RenderStageLabel::Extract,
            SystemContainer::new(PopulateWorldSystem::<E, T>::new(&kernel)),
        );

        schedule.add_system_to_stage(RenderStageLabel::Prepare, resource_system);
        schedule.add_system_to_stage(RenderStageLabel::Queue, upload_system); // FIXME tcs: Upload updates the TileView in tileviewpattern -> upload most run before prepare
        schedule.add_system_to_stage(RenderStageLabel::Queue, queue_system);
    }
}

pub struct AvailableVectorLayerData {
    pub coords: WorldTileCoords,
    pub source_layer: String,
    pub buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
    /// Holds for each feature the count of indices.
    pub feature_indices: Vec<u32>,
}

pub struct MissingVectorLayerData {
    pub coords: WorldTileCoords,
    pub source_layer: String,
}

pub enum VectorLayerData {
    Available(AvailableVectorLayerData),
    Missing(MissingVectorLayerData),
}

#[derive(Default)]
pub struct VectorLayersDataComponent {
    pub done: bool,
    pub layers: Vec<VectorLayerData>,
}

impl TileComponent for VectorLayersDataComponent {}
