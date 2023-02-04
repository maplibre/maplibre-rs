mod phase_sort;
mod populate_world_system;
mod prepare;
mod request;
mod resource;
mod tile_view_pattern;
mod upload;

use std::ops::{Deref, DerefMut};

use crate::{
    ecs::world::World,
    environment::Environment,
    kernel::Kernel,
    plugin::Plugin,
    render::{
        eventually::Eventually,
        render_phase::RenderPhase,
        resource::{BufferPool, Globals, IndexEntry, RasterResources},
        shaders::{ShaderFeatureStyle, ShaderLayerMetadata},
        tile_view_pattern::{TileShape, TileViewPattern},
        ShaderVertex,
    },
    tessellation::IndexDataType,
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

#[derive(Default)]
pub struct MaskRenderPhase(RenderPhase<TileShape>);
impl Deref for MaskRenderPhase {
    type Target = RenderPhase<TileShape>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for MaskRenderPhase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Default)]
pub struct RasterTilePhase(RenderPhase<TileShape>);
impl Deref for RasterTilePhase {
    type Target = RenderPhase<TileShape>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for RasterTilePhase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Default)]
pub struct VectorTilePhase(RenderPhase<(IndexEntry, TileShape)>);
impl Deref for VectorTilePhase {
    type Target = RenderPhase<(IndexEntry, TileShape)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for VectorTilePhase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
    fn build(&self, kernel: &mut Kernel<E>, world: &mut World) {
        // FIXME: Split into several plugins

        // globals_bind_group
        world.insert_resource(Eventually::<Globals>::Uninitialized);

        // buffer_pool
        world.insert_resource(Eventually::<VectorBufferPool>::Uninitialized);

        // tile_view_pattern:
        world.insert_resource(
            // FIXME: Simplify type
            Eventually::<TileViewPattern<wgpu::Queue, wgpu::Buffer>>::Uninitialized,
        );

        // raster_resources
        world.insert_resource(Eventually::<RasterResources>::Uninitialized);

        // vector_tile_pipeline
        world.insert_resource(Eventually::<VectorPipeline>::Uninitialized);
        // mask_pipeline
        world.insert_resource(Eventually::<MaskPipeline>::Uninitialized);
        // debug_pipeline
        world.insert_resource(Eventually::<DebugPipeline>::Uninitialized);

        // mask_phase
        world.insert_resource(MaskRenderPhase::default());
        // vector_tile_phase
        world.insert_resource(VectorTilePhase::default());
        // raster_tile_phase,
        world.insert_resource(RasterTilePhase::default());
    }
}
