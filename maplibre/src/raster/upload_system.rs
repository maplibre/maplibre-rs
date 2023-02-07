//! Uploads data to the GPU which is needed for rendering.
use crate::{
    context::MapContext,
    coords::ViewRegion,
    io::tile_repository::{StoredLayer, TileRepository},
    render::{
        eventually::{Eventually, Eventually::Initialized},
        resource::RasterResources,
        Renderer,
    },
    style::Style,
};

pub fn upload_system(
    MapContext {
        world,
        style,
        renderer: Renderer { device, queue, .. },
        ..
    }: &mut MapContext,
) {
    let view_state = &world.view_state;
    let view_region = view_state.create_view_region();

    let Initialized(raster_resources) = world.resources.get_mut::<
        Eventually<RasterResources>
    >().unwrap() else {
        return; };

    if let Some(view_region) = &view_region {
        upload_raster_layer(
            raster_resources,
            device,
            queue,
            &world.tile_repository,
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
    tile_repository: &TileRepository,
    style: &Style,
    view_region: &ViewRegion,
) {
    for coords in view_region.iter() {
        if raster_resources.get_bound_texture(&coords).is_some() {
            continue;
        }

        let Some(raster_layers) =
            tile_repository.iter_raster_layers_at(raster_resources, &coords) else { continue; };

        for style_layer in &style.layers {
            let source_layer = style_layer.source_layer.as_ref().unwrap(); // TODO: Remove unwrap

            let Some(stored_layer) = raster_layers
                .iter()
                .find(|layer| source_layer.as_str() == layer.layer_name()) else { continue; };

            match stored_layer {
                StoredLayer::UnavailableLayer { .. } => {}
                StoredLayer::TessellatedLayer { .. } => {}
                StoredLayer::RasterLayer {
                    coords,
                    layer_name,
                    image,
                } => {
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
                        &image,
                        wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: std::num::NonZeroU32::new(4 * width),
                            rows_per_image: std::num::NonZeroU32::new(height),
                        },
                        texture.size.clone(),
                    );

                    raster_resources.bind_texture(device, &coords, texture);
                }
            }
        }
    }
}
