use std::collections::HashMap;

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
    pub sampler: Option<wgpu::Sampler>,
    pub msaa: Option<Msaa>,
    pub texture: Option<Texture>,
    pub pipeline: Option<wgpu::RenderPipeline>,
    pub bind_groups: HashMap<WorldTileCoords, wgpu::BindGroup>,
}

impl RasterResources {
    pub fn set_msaa(&mut self, msaa: Msaa) {
        self.msaa = Some(msaa);
    }

    pub fn set_texture(
        &mut self,
        label: wgpu::Label,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        usage: wgpu::TextureUsages,
    ) {
        self.texture = Some(Texture::new(
            label,
            device,
            format,
            width,
            height,
            self.msaa.unwrap().clone(),
            usage,
        ));
    }

    pub fn set_sampler(&mut self, device: &wgpu::Device) {
        self.sampler = Some(device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        }));
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

    /// Creates a bind group for each fetched raster tile and store it inside a hashmap.
    pub fn set_raster_bind_group(&mut self, device: &wgpu::Device, coords: &WorldTileCoords) {
        self.bind_groups.insert(
            *coords,
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.pipeline.as_ref().unwrap().get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &self.texture.as_ref().unwrap().view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(self.sampler.as_ref().unwrap()),
                    },
                ],
                label: None,
            }),
        );
    }
}

impl HasTile for RasterResources {
    fn has_tile(&self, coords: &WorldTileCoords) -> bool {
        self.bind_groups.contains_key(coords)
    }
}

impl Default for RasterResources {
    fn default() -> Self {
        RasterResources {
            sampler: None,
            msaa: None,
            texture: None,
            pipeline: None,
            bind_groups: HashMap::new(),
        }
    }
}
