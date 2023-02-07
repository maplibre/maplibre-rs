//! Uploads data to the GPU which is needed for rendering.

use std::iter;

use crate::{
    context::MapContext,
    coords::{ViewRegion, WorldTileCoords},
    io::tile_repository::{StoredLayer, TileRepository},
    render::{
        camera::ViewProjection,
        eventually::{Eventually, Eventually::Initialized},
        resource::IndexEntry,
        shaders::{ShaderFeatureStyle, ShaderLayerMetadata, Vec4f32},
        tile_view_pattern::TileViewPattern,
        Renderer,
    },
    style::Style,
    vector::{VectorBufferPool, VectorLayersComponent},
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
    let view_proj = view_state.view_projection();
    let view_region = view_state.create_view_region();

    // TODO duplicate
    let (
        Initialized(tile_view_pattern),
        Initialized(buffer_pool),
    ) = world.resources.collect_mut2::<
        Eventually<TileViewPattern<wgpu::Queue, wgpu::Buffer>>,
        Eventually<VectorBufferPool>,
    >().unwrap() else {
        return; };

    if let Some(view_region) = &view_region {
        let new_components = upload_tesselated_layer(
            buffer_pool,
            device,
            queue,
            &world.tile_repository,
            style,
            view_region,
        );
        upload_tile_view_pattern(tile_view_pattern, queue, &view_proj);

        for (coords, components) in new_components {
            world
                .query_tile_mut(coords)
                .unwrap()
                .insert(VectorLayersComponent {
                    entries: components,
                });
        }
        //self.update_metadata(state, tile_repository, queue);
    }
}

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

fn upload_tile_view_pattern(
    tile_view_pattern: &mut TileViewPattern<wgpu::Queue, wgpu::Buffer>,
    queue: &wgpu::Queue,
    view_proj: &ViewProjection,
) {
    tile_view_pattern.upload_pattern(queue, view_proj);
}

fn upload_tesselated_layer(
    buffer_pool: &mut VectorBufferPool,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    tile_repository: &TileRepository,
    style: &Style,
    view_region: &ViewRegion,
) -> Vec<(WorldTileCoords, Vec<IndexEntry>)> {
    let mut new_components_tiles = Vec::new();

    // Upload all tessellated layers which are in view
    for coords in view_region.iter() {
        let Some(available_layers) =
            tile_repository.iter_missing_tesselated_layers_at(buffer_pool, &coords) else { continue; };

        let mut new_components = Vec::new();

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
                    let entry = buffer_pool.allocate_layer_geometry(
                        queue,
                        *coords,
                        style_layer.clone(),
                        buffer,
                        ShaderLayerMetadata::new(style_layer.index as f32),
                        &feature_metadata,
                    );

                    new_components.push(entry);
                }
            }
        }

        new_components_tiles.push((coords, new_components));
    }

    return new_components_tiles;
}
