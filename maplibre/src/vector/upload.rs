//! Uploads data to the GPU which is needed for rendering.

use std::iter;

use crate::{
    context::MapContext,
    coords::ViewRegion,
    ecs::world::World,
    io::tile_repository::{StoredLayer, TileRepository},
    render::{
        camera::ViewProjection,
        eventually::{Eventually, Eventually::Initialized},
        resource::{Globals, RasterResources},
        shaders::{ShaderCamera, ShaderFeatureStyle, ShaderGlobals, ShaderLayerMetadata, Vec4f32},
        tile_view_pattern::TileViewPattern,
        Renderer,
    },
    style::Style,
    vector::VectorBufferPool,
};

fn upload_system(
    MapContext {
        world,
        style,
        renderer: Renderer { device, queue, .. },
        ..
    }: &mut MapContext,
) {
    // TODO duplicate
    let (Initialized(tile_view_pattern), Initialized(buffer_pool), Initialized(raster_resources)) =
        (
            world.get_resource_mut::<Eventually<TileViewPattern<wgpu::Queue, wgpu::Buffer>>>(),
            world.get_resource_mut::<Eventually<VectorBufferPool>>(),
            world.get_resource_mut::<Eventually<RasterResources>>(),
        ) else { return; };

    let view_state = &world.view_state;
    let tile_repository = &world.tile_repository;
    let view_proj = view_state.view_projection();

    if let Initialized(globals_bind_group) = &world.get_resource_mut::<Eventually<Globals>>() {
        // Update globals
        queue.write_buffer(
            &globals_bind_group.uniform_buffer,
            0,
            bytemuck::cast_slice(&[ShaderGlobals::new(ShaderCamera::new(
                view_proj.downcast().into(),
                view_state
                    .camera()
                    .position()
                    .to_homogeneous()
                    .cast::<f32>()
                    .unwrap() // TODO: Remove unwrap
                    .into(),
            ))]),
        );
    }

    let view_region = view_state.create_view_region();

    if let Some(view_region) = &view_region {
        upload_tesselated_layer(
            buffer_pool,
            device,
            queue,
            tile_repository,
            style,
            view_region,
        );
        upload_raster_layer(
            raster_resources,
            device,
            queue,
            tile_repository,
            style,
            view_region,
        );
        upload_tile_view_pattern(tile_view_pattern, queue, &view_proj);
        //self.update_metadata(state, tile_repository, queue);
    }
}

#[tracing::instrument(skip_all)]
fn update_metadata(
    buffer_pool: &VectorBufferPool,
    tile_repository: &TileRepository,
    queue: &wgpu::Queue,
) {
    let animated_one = 0.5
        * (1.0
            + ((std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64())
            .sin()));

    for entries in buffer_pool.index().iter() {
        for entry in entries {
            let world_coords = entry.coords;

            let source_layer = entry.style_layer.source_layer.as_ref().unwrap();

            let Some(stored_layer) =
                tile_repository
                    .iter_layers_at(&world_coords)
                    .and_then(|mut layers| {
                        layers.find(|layer| source_layer.as_str() == layer.layer_name())
                    })  else { continue; };

            let color: Option<Vec4f32> = entry
                .style_layer
                .paint
                .as_ref()
                .and_then(|paint| paint.get_color())
                .map(|mut color| {
                    color.color.b = animated_one as f32;
                    color.into()
                });

            match stored_layer {
                StoredLayer::UnavailableLayer { .. } => {}
                StoredLayer::TessellatedLayer {
                    feature_indices, ..
                } => {
                    /* let feature_metadata = layer_data
                    .features()
                    .iter()
                    .enumerate()
                    .flat_map(|(i, _feature)| {
                        iter::repeat(ShaderFeatureStyle {
                            color: color.unwrap(),
                        })
                        .take(feature_indices[i] as usize)
                    })
                    .collect::<Vec<_>>();*/

                    let feature_metadata = (0..feature_indices.len())
                        .flat_map(|i| {
                            iter::repeat(ShaderFeatureStyle {
                                color: color.unwrap(),
                            })
                            .take(feature_indices[i] as usize)
                        })
                        .collect::<Vec<_>>();

                    buffer_pool.update_feature_metadata(queue, entry, &feature_metadata);
                }

                StoredLayer::RasterLayer { .. } => {}
            }
        }
    }
}

#[tracing::instrument(skip_all)]
fn upload_tile_view_pattern(
    tile_view_pattern: &mut TileViewPattern<wgpu::Queue, wgpu::Buffer>,
    queue: &wgpu::Queue,
    view_proj: &ViewProjection,
) {
    tile_view_pattern.upload_pattern(queue, view_proj);
}

#[tracing::instrument(skip_all)]
fn upload_tesselated_layer(
    buffer_pool: &mut VectorBufferPool,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    tile_repository: &TileRepository,
    style: &Style,
    view_region: &ViewRegion,
) {
    // Upload all tessellated layers which are in view
    // FIXME: Take into account raster layers
    for coords in view_region.iter() {
        let Some(available_layers) =
            tile_repository.iter_missing_tesselated_layers_at(buffer_pool, &coords) else { continue; };

        for style_layer in &style.layers {
            let source_layer = style_layer.source_layer.as_ref().unwrap(); // TODO: Remove unwrap

            let Some(stored_layer) = available_layers
                .iter()
                .find(|layer| source_layer.as_str() == layer.layer_name()) else { continue; };

            let color: Option<Vec4f32> = style_layer
                .paint
                .as_ref()
                .and_then(|paint| paint.get_color())
                .map(|color| color.into());

            match stored_layer {
                StoredLayer::UnavailableLayer { .. } => {}
                StoredLayer::RasterLayer { .. } => {}
                StoredLayer::TessellatedLayer {
                    coords,
                    feature_indices,
                    buffer,
                    ..
                } => {
                    let allocate_feature_metadata =
                        tracing::span!(tracing::Level::TRACE, "allocate_feature_metadata");

                    let guard = allocate_feature_metadata.enter();
                    let feature_metadata = (0..feature_indices.len()) // FIXME: Iterate over actual featrues
                        .enumerate()
                        .flat_map(|(i, _feature)| {
                            iter::repeat(ShaderFeatureStyle {
                                color: color.unwrap(),
                            })
                            .take(feature_indices[i] as usize)
                        })
                        .collect::<Vec<_>>();
                    drop(guard);

                    tracing::trace!("Allocating geometry at {}", &coords);
                    log::info!("Allocating geometry at {}", &coords);
                    buffer_pool.allocate_layer_geometry(
                        queue,
                        *coords,
                        style_layer.clone(),
                        buffer,
                        ShaderLayerMetadata::new(style_layer.index as f32),
                        &feature_metadata,
                    );
                }
            }
        }
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
        let Some(available_layers) =
            tile_repository.iter_missing_raster_layers_at(raster_resources, &coords) else { continue; };

        for style_layer in &style.layers {
            let source_layer = style_layer.source_layer.as_ref().unwrap(); // TODO: Remove unwrap

            let Some(stored_layer) = available_layers
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
