use std::{marker::PhantomData, ops::Deref, rc::Rc};
use std::ops::Range;
use lyon::geom::Box2D;

use crate::{
    coords::WorldTileCoords,
    environment::Environment,
    kernel::Kernel,
    plugin::Plugin,
    render::{
        eventually::Eventually,
        graph::RenderGraph,
        shaders::{ShaderLayerMetadata, ShaderSymbolVertex},
        RenderStageLabel,
    },
    schedule::Schedule,
    sdf::resource::GlyphTexture,
    tcs::{system::SystemContainer, tiles::TileComponent, world::World},
    vector::{
        resource::BufferPool,
        tessellation::{IndexDataType, OverAlignedVertexBuffer},
        VectorTransferables,
    },
};
use crate::render::shaders::SDFShaderFeatureMetadata;

mod populate_world_system;
mod queue_system;
mod render_commands;
mod resource;
mod resource_system;
mod upload_system;

// Public due to bechmarks
pub mod tessellation;
mod text;

struct SymbolPipeline(wgpu::RenderPipeline);

impl Deref for SymbolPipeline {
    type Target = wgpu::RenderPipeline;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type SymbolBufferPool = BufferPool<
    wgpu::Queue,
    wgpu::Buffer,
    ShaderSymbolVertex,
    IndexDataType,
    ShaderLayerMetadata,
    SDFShaderFeatureMetadata,
>;

pub struct SdfPlugin<T>(PhantomData<T>);

impl<T: VectorTransferables> Default for SdfPlugin<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<E: Environment, T: VectorTransferables> Plugin<E> for SdfPlugin<T> {
    fn build(
        &self,
        schedule: &mut Schedule,
        kernel: Rc<Kernel<E>>,
        world: &mut World,
        _graph: &mut RenderGraph,
    ) {
        let resources = &mut world.resources;

        resources.insert(Eventually::<SymbolPipeline>::Uninitialized);
        resources.insert(Eventually::<SymbolBufferPool>::Uninitialized);
        resources.insert(Eventually::<GlyphTexture>::Uninitialized);
        resources.insert(Eventually::<(wgpu::Texture, wgpu::Sampler)>::Uninitialized);

        schedule.add_system_to_stage(
            RenderStageLabel::Extract,
            SystemContainer::new(
                populate_world_system::PopulateWorldSystem::<E, T>::new(&kernel),
            ),
        );

        schedule.add_system_to_stage(
            RenderStageLabel::Prepare,
            resource_system::resource_system,
        );
        schedule.add_system_to_stage(
            RenderStageLabel::Queue,
            upload_system::upload_system,
        ); // FIXME tcs: Upload updates the TileView in tileviewpattern -> upload most run before prepare
        schedule.add_system_to_stage(
            RenderStageLabel::Queue,
            queue_system::queue_system,
        );
    }
}

pub struct Feature {
    pub bbox: Box2D<f32>,
    pub indices: Range<usize>,
}

pub struct SymbolLayerData {
    pub coords: WorldTileCoords,
    pub source_layer: String,
    pub buffer: OverAlignedVertexBuffer<ShaderSymbolVertex, IndexDataType>,
    pub features: Vec<Feature>
}

#[derive(Default)]
pub struct SymbolLayersDataComponent {
    pub layers: Vec<SymbolLayerData>,
}

impl TileComponent for SymbolLayersDataComponent {}
