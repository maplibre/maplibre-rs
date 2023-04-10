//! Uploads data to the GPU which is needed for rendering.
use crate::{
    context::MapContext,
    coords::ViewRegion,
    raster::{
        resource::RasterResources, AvailableRasterLayerData, RasterLayerData,
        RasterLayersDataComponent,
    },
    render::{
        eventually::{Eventually, Eventually::Initialized},
        Renderer,
    },
    style::Style,
    tcs::tiles::Tiles,
};

pub fn upload_system(
    MapContext {
        world,
        style,
        view_state,
        renderer: Renderer { device, queue, .. },
        ..
    }: &mut MapContext,
) {
    let Some(Initialized(raster_resources)) = world
        .resources
        .query_mut::<&mut Eventually<RasterResources>>() else { return; };
    let view_region = view_state.create_view_region();

    if let Some(view_region) = &view_region {
        upload_raster_layer(
            raster_resources,
            device,
            queue,
            &world.tiles,
            style,
            view_region,
        );
    }
}

#[tracing::instrument(skip_all)]
fn upload_raster_layer(
    raster_resources: &mut RasterResources,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    tiles: &Tiles,
    style: &Style,
    view_region: &ViewRegion,
) {
    for coords in view_region.iter() {
        if raster_resources.get_bound_texture(&coords).is_some() {
            continue;
        }

        let Some(raster_layers) =
            tiles.query::<&RasterLayersDataComponent>(coords) else { continue; };

        for style_layer in &style.layers {
            let style_source_layer = style_layer.source_layer.as_ref().unwrap(); // FIXME: Remove unwrap

            let Some(AvailableRasterLayerData {
                coords,
                image,
                ..
            }) = raster_layers.layers
                .iter()
                .flat_map(|data| match data {
                    RasterLayerData::Available(data) => Some(data),
                    RasterLayerData::Missing(_) => None,
                })
                .find(|layer| style_source_layer.as_str() == layer.source_layer) else { continue; };

            let (width, height) = image.dimensions();

            let texture = raster_resources.create_texture(
                None,
                device,
                wgpu::TextureFormat::Rgba8UnormSrgb,
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
}
