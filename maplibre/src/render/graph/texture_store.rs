//! Rendergraph

use std::collections::HashMap;
use wgpu::{Device, Extent3d, Texture, TextureDescriptor, TextureDimension};

use super::RenderTargetCore;

struct StoredTexture {
    inner: Texture,
    used: bool,
}

pub(crate) struct GraphTextureStore {
    textures: HashMap<RenderTargetCore, Vec<StoredTexture>>,
}
impl GraphTextureStore {
    pub fn new() -> Self {
        Self {
            textures: HashMap::with_capacity(32),
        }
    }

    pub fn get_texture(&mut self, device: &Device, desc: RenderTargetCore) -> Texture {
        let vec = self
            .textures
            .entry(desc)
            .or_insert_with(|| Vec::with_capacity(16));
        if let Some(tex) = vec.pop() {
            return tex.inner;
        }

        device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: desc.resolution.x,
                height: desc.resolution.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: desc.samples as _,
            dimension: TextureDimension::D2,
            format: desc.format,
            usage: desc.usage,
        })
    }

    pub fn return_texture(&mut self, desc: RenderTargetCore, tex: Texture) {
        let vec = self
            .textures
            .entry(desc)
            .or_insert_with(|| Vec::with_capacity(16));

        vec.push(StoredTexture {
            inner: tex,
            used: true,
        });
    }

    pub fn mark_unused(&mut self) {
        for vec in self.textures.values_mut() {
            for tex in vec {
                tex.used = false;
            }
        }
    }

    pub fn remove_unused(&mut self) {
        for vec in self.textures.values_mut() {
            vec.retain(|t| t.used);
        }

        self.textures.retain(|_, v| !v.is_empty());
    }
}
