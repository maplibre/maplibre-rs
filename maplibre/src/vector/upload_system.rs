//! Uploads data to the GPU which is needed for rendering.

use std::iter;

use crate::{
    context::MapContext,
    coords::ViewRegion,
    render::{
        eventually::{Eventually, Eventually::Initialized},
        shaders::{ShaderFeatureStyle, ShaderLayerMetadata, Vec4f32},
        Renderer,
    },
    style::Style,
    tcs::tiles::Tiles,
    vector::{
        AvailableVectorLayerData, VectorBufferPool, VectorLayerData, VectorLayersDataComponent,
    },
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
    let Some(
        Initialized(buffer_pool)
    ) = world.resources.query_mut::<
        &mut Eventually<VectorBufferPool>,
    >() else { return; };

    let view_region = view_state.create_view_region();

    if let Some(view_region) = &view_region {
        upload_tesselated_layer(
            buffer_pool,
            device,
            queue,
            &mut world.tiles,
            style,
            view_region,
        );
        // self.update_metadata(state, tile_repository, queue);
    }
}

/* FIXME tcs fn update_metadata(
    buffer_pool: &VectorBufferPool,
    tiles: &Tiles,
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
}*/

fn upload_tesselated_layer(
    buffer_pool: &mut VectorBufferPool,
    _device: &wgpu::Device,
    queue: &wgpu::Queue,
    tiles: &mut Tiles,
    style: &Style,
    view_region: &ViewRegion,
) {
    // Upload all tessellated layers which are in view
    for coords in view_region.iter() {
        let Some(vector_layers) = tiles.query_mut::<&VectorLayersDataComponent>(coords) else { continue; };

        let loaded_layers = buffer_pool
            .get_loaded_source_layers_at(coords)
            .unwrap_or_default();

        let available_layers = vector_layers
            .layers
            .iter()
            .flat_map(|data| match data {
                VectorLayerData::Available(data) => Some(data),
                VectorLayerData::Missing(_) => None,
            })
            .filter(|data| !loaded_layers.contains(data.source_layer.as_str()))
            .collect::<Vec<_>>();

        for style_layer in &style.layers {
            let source_layer = style_layer.source_layer.as_ref().unwrap(); // TODO: Unwrap

            let Some(AvailableVectorLayerData {
                         coords,
                         feature_indices,
                         buffer,
                         ..
                     }) = available_layers
                .iter()
                .find(|layer| source_layer.as_str() == layer.source_layer) else { continue; };

            let color: Option<Vec4f32> = style_layer
                .paint
                .as_ref()
                .and_then(|paint| paint.get_color())
                .map(|color| color.into());

            let feature_metadata = (0..feature_indices.len()) // FIXME: Iterate over actual features
                .enumerate()
                .flat_map(|(i, _feature)| {
                    iter::repeat(ShaderFeatureStyle {
                        color: color.unwrap(),
                    })
                    .take(feature_indices[i] as usize)
                })
                .collect::<Vec<_>>();

            log::debug!("Allocating geometry at {coords}");
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
