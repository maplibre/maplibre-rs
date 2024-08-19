//! Uploads data to the GPU which is needed for rendering.

use std::iter;

use crate::{
    context::MapContext,
    coords::ViewRegion,
    render::{
        eventually::{Eventually, Eventually::Initialized},
        shaders::{FillShaderFeatureMetadata, ShaderLayerMetadata, Vec4f32},
        tile_view_pattern::DEFAULT_TILE_SIZE,
        view_state::ViewStatePadding,
        Renderer,
    },
    style::Style,
    tcs::{
        system::{SystemError, SystemResult},
        tiles::Tiles,
    },
    vector::{
        AvailableVectorLayerBucket, VectorBufferPool, VectorLayerBucket, VectorLayerBucketComponent,
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
) -> SystemResult {
    let Some(Initialized(buffer_pool)) = world
        .resources
        .query_mut::<&mut Eventually<VectorBufferPool>>()
    else {
        return Err(SystemError::Dependencies);
    };

    let view_region = view_state.create_view_region(
        view_state.zoom().zoom_level(DEFAULT_TILE_SIZE),
        ViewStatePadding::Loose,
    );

    if let Some(view_region) = &view_region {
        upload_tessellated_layer(
            buffer_pool,
            device,
            queue,
            &mut world.tiles,
            style,
            view_region,
        );
    }

    Ok(())
}

fn upload_tessellated_layer(
    buffer_pool: &mut VectorBufferPool,
    _device: &wgpu::Device,
    queue: &wgpu::Queue,
    tiles: &mut Tiles,
    style: &Style,
    view_region: &ViewRegion,
) {
    // Upload all tessellated layers which are in view
    for coords in view_region.iter() {
        let Some(vector_layers) = tiles.query_mut::<&VectorLayerBucketComponent>(coords) else {
            continue;
        };

        let loaded_layers = buffer_pool
            .get_loaded_source_layers_at(coords)
            .unwrap_or_default();

        let available_layers = vector_layers
            .layers
            .iter()
            .flat_map(|data| match data {
                VectorLayerBucket::AvailableLayer(data) => Some(data),
                VectorLayerBucket::Missing(_) => None,
            })
            .filter(|data| !loaded_layers.contains(data.source_layer.as_str()))
            .collect::<Vec<_>>();

        for style_layer in &style.layers {
            let layer_id = &style_layer.id;
            let source_layer = match style_layer.source_layer.as_ref() {
                Some(layer) => layer,
                None => {
                    log::trace!("style layer {layer_id} does not have a source layer");
                    continue;
                }
            };

            let Some(AvailableVectorLayerBucket {
                coords,
                feature_indices,
                buffer,
                ..
            }) = available_layers
                .iter()
                .find(|layer| source_layer.as_str() == layer.source_layer)
            else {
                continue;
            };

            let color: Option<Vec4f32> = style_layer
                .paint
                .as_ref()
                .and_then(|paint| paint.get_color())
                .map(|color| color.into());

            // Assign every feature in the layer the color from the style
            let feature_metadata = iter::repeat(FillShaderFeatureMetadata {
                color: color.unwrap(),
            })
                .take(feature_indices.iter().sum::<u32>() as usize)
                .collect::<Vec<_>>();

            // FIXME avoid uploading empty indices
            if buffer.buffer.indices.is_empty() {
                continue;
            }

            log::debug!("Allocating geometry at {coords}");
            buffer_pool.allocate_layer_geometry(
                queue,
                *coords,
                style_layer.clone(),
                buffer,
                ShaderLayerMetadata {
                    z_index: style_layer.index as f32,
                },
                &feature_metadata,
            );
        }
    }
}
