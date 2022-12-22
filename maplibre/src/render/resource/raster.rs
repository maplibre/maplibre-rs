use std::collections::HashMap;

use wgpu::BindGroup;

use crate::{
    coords::WorldTileCoords,
    render::{
        resource::{RenderPipeline, Texture},
        settings::{Msaa, RendererSettings},
        shaders::{RasterTileShader, Shader},
        tile_pipeline::TilePipeline,
        tile_view_pattern::HasTile,
    },
};

/// Holds the resources necessary for the raster tiles such as the
/// * sampler
/// * texture
/// * pipeline
/// * bindgroups
pub struct RasterResources {
    sampler: wgpu::Sampler,
    msaa: Msaa,
    pipeline: Option<wgpu::RenderPipeline>,
    bind_groups: HashMap<WorldTileCoords, wgpu::BindGroup>,
}

impl RasterResources {
    pub fn new(msaa: Msaa, device: &wgpu::Device) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self {
            sampler,
            msaa,
            pipeline: None,
            bind_groups: Default::default(),
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
        Texture::new(
            label,
            device,
            format,
            width,
            height,
            self.msaa.clone(),
            usage,
        )
    }

    pub fn set_raster_pipeline(
        &mut self,
        device: &wgpu::Device,
        settings: &RendererSettings,
        shader: &RasterTileShader,
    ) {
        self.pipeline = Some(
            TilePipeline::new(
                *settings,
                shader.describe_vertex(),
                shader.describe_fragment(),
                false,
                true,
                false,
                false,
                false,
                true,
                true,
            )
            .describe_render_pipeline()
            .initialize(device),
        );
    }

    pub fn get_bind_group(&self, coords: &WorldTileCoords) -> Option<&BindGroup> {
        self.bind_groups.get(coords)
    }

    /// Creates a bind group for each fetched raster tile and store it inside a hashmap.
    pub fn bind_texture(
        &mut self,
        device: &wgpu::Device,
        coords: &WorldTileCoords,
        texture: Texture,
    ) {
        self.bind_groups.insert(
            *coords,
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.pipeline.as_ref().unwrap().get_bind_group_layout(0),
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

    pub fn pipeline(&self) -> &Option<wgpu::RenderPipeline> {
        &self.pipeline
    }
}

impl HasTile for RasterResources {
    fn has_tile(&self, coords: &WorldTileCoords) -> bool {
        self.bind_groups.contains_key(coords)
    }
}
