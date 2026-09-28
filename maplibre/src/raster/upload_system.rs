//! Uploads data to the GPU which is needed for rendering.
use std::collections::BTreeSet;

use crate::{
    context::MapContext,
    raster::{
        resource::RasterResources, AvailableRasterLayerData, RasterLayerData,
        RasterLayersDataComponent,
    },
    render::{
        eventually::{Eventually, Eventually::Initialized},
        tile_view_pattern::WgpuTileViewPattern,
        Renderer,
    },
    tcs::{
        system::{SystemError, SystemResult},
        tiles::Tiles,
    },
};

pub fn upload_system(
    MapContext {
        world,
        renderer: Renderer { device, queue, .. },
        ..
    }: &mut MapContext,
) -> SystemResult {
    let Some((Initialized(raster_resources), Initialized(tile_view_pattern))) =
        world.resources.query_mut::<(
            &mut Eventually<RasterResources>,
            &Eventually<WgpuTileViewPattern>,
        )>()
    else {
        return Err(SystemError::Dependencies);
    };

    let mut source_tiles = BTreeSet::new();
    for view_tile in tile_view_pattern.iter() {
        view_tile.render(|shape| {
            source_tiles.insert(shape.coords());
        });
    }
    upload_raster_layer(raster_resources, device, queue, &world.tiles, source_tiles);

    Ok(())
}

#[tracing::instrument(skip_all)]
fn upload_raster_layer(
    raster_resources: &mut RasterResources,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    tiles: &Tiles,
    source_tiles: BTreeSet<crate::coords::WorldTileCoords>,
) {
    for coords in source_tiles {
        if raster_resources.get_bound_texture(&coords).is_some() {
            continue;
        }

        let Some(raster_layers) = tiles.query::<&RasterLayersDataComponent>(coords) else {
            continue;
        };

        let Some(AvailableRasterLayerData { coords, image, .. }) =
            raster_layers.layers.iter().find_map(|data| match data {
                RasterLayerData::Available(data) => Some(data),
                RasterLayerData::Missing(_) => None,
            })
        else {
            continue;
        };

        let (width, height) = image.dimensions();

        let texture = raster_resources.create_texture(
            None,
            device,
            // Raster style colors are sampled in the encoded color space, matching WebGL's
            // default RGBA upload path. An sRGB texture view would decode the texels to linear
            // values before writing them to the non-sRGB render target, making imagery too dark.
            wgpu::TextureFormat::Rgba8Unorm,
            width,
            height,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            image,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            texture.size,
        );

        raster_resources.bind_texture(device, coords, texture);
    }
}
