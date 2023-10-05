use std::collections::HashMap;

use crate::{
    coords::WorldTileCoords,
    render::{resource::Texture, settings::Msaa, tile_view_pattern::HasTile},
    tcs::world::World,
};

/// Holds the resources necessary for the raster tiles such as the
/// * sampler
/// * texture
/// * pipeline
/// * bindgroups
pub struct RasterResources {
    sampler: wgpu::Sampler,
    msaa: Msaa,
    pipeline: wgpu::RenderPipeline,
    bound_textures: HashMap<WorldTileCoords, wgpu::BindGroup>,
}

impl RasterResources {
    pub fn new(msaa: Msaa, device: &wgpu::Device, pipeline: wgpu::RenderPipeline) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Self {
            sampler,
            msaa,
            pipeline,
            bound_textures: Default::default(),
        }
    }

    pub fn create_texture(
        &mut self,
        label: wgpu::Label,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        usage: wgpu::TextureUsages,
    ) -> Texture {
        Texture::new(label, device, format, width, height, self.msaa, usage)
    }

    pub fn get_bound_texture(&self, coords: &WorldTileCoords) -> Option<&wgpu::BindGroup> {
        self.bound_textures.get(coords)
    }

    /// Creates a bind group for each fetched raster tile and store it inside a hashmap.
    pub fn bind_texture(
        &mut self,
        device: &wgpu::Device,
        coords: &WorldTileCoords,
        texture: Texture,
    ) {
        self.bound_textures.insert(
            *coords,
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.pipeline.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
                label: None,
            }),
        );
    }

    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}

impl HasTile for RasterResources {
    fn has_tile(&self, coords: WorldTileCoords, _world: &World) -> bool {
        self.bound_textures.contains_key(&coords)
    }
}
