use std::{
    marker::PhantomData,
    ops::{Deref, Range},
    rc::Rc,
};

use crate::euclid::{Box2D, Point2D};
use crate::{
    coords::WorldTileCoords,
    environment::Environment,
    kernel::Kernel,
    plugin::Plugin,
    render::{
        eventually::Eventually,
        graph::RenderGraph,
        RenderStageLabel,
        shaders::{SDFShaderFeatureMetadata, ShaderLayerMetadata, ShaderSymbolVertex},
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

mod populate_world_system;
mod queue_system;
mod render_commands;
mod resource;
mod resource_system;
mod upload_system;

// Public due to bechmarks
mod bidi;
mod buckets;
mod collision_feature;
mod collision_index;
mod collision_system;
mod font_stack;
mod geometry;
mod geometry_tile_data;
mod glyph;
mod glyph_atlas;
mod glyph_range;
mod grid_index;
mod image;
mod image_atlas;
mod layout;
mod shaping;
mod style_types;
mod tagged_string;
pub mod tessellation;
mod text;
mod util;

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
            SystemContainer::new(populate_world_system::PopulateWorldSystem::<E, T>::new(
                &kernel,
            )),
        );

        schedule.add_system_to_stage(RenderStageLabel::Prepare, resource_system::resource_system);
        schedule.add_system_to_stage(RenderStageLabel::Queue, upload_system::upload_system); // FIXME tcs: Upload updates the TileView in tileviewpattern -> upload most run before prepare
        schedule.add_system_to_stage(RenderStageLabel::Queue, queue_system::queue_system);

        schedule.add_system_to_stage(
            RenderStageLabel::PhaseSort,
            SystemContainer::new(collision_system::CollisionSystem::new()),
        );
    }
}

pub struct Feature {
    pub bbox: Box2D<f32, TileSpace>,
    pub indices: Range<usize>,
    pub text_anchor: Point2D<f32, TileSpace>,
    pub str: String,
}

pub struct SymbolLayerData {
    pub coords: WorldTileCoords,
    pub source_layer: String,
    pub buffer: OverAlignedVertexBuffer<ShaderSymbolVertex, IndexDataType>,
    pub features: Vec<Feature>,
}

#[derive(Default)]
pub struct SymbolLayersDataComponent {
    pub layers: Vec<SymbolLayerData>,
}

impl TileComponent for SymbolLayersDataComponent {}

// TODO where should this live?
pub struct TileSpace; // The unit in which geometries or symbols are on a tile (0-EXTENT)
pub struct ScreenSpace;

// TODO where should this live?
#[derive(PartialEq)]
pub enum MapMode {
    ///< continually updating map
    Continuous,
    ///< a once-off still image of an arbitrary viewport
    Static,
    ///< a once-off still image of a single tile
    Tile,
}
