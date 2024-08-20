//! Uploads data to the GPU which is needed for rendering.

use std::iter;

use crate::{
    context::MapContext,
    coords::ViewRegion,
    render::{
        eventually::{Eventually, Eventually::Initialized},
        shaders::ShaderLayerMetadata,
        tile_view_pattern::DEFAULT_TILE_SIZE,
        view_state::ViewStatePadding,
        Renderer,
    },
    sdf::{SymbolBufferPool, SymbolLayerData, SymbolLayersDataComponent},
    style::Style,
    tcs::{
        system::{SystemError, SystemResult},
        tiles::Tiles,
    },
};
use crate::render::shaders::{FillShaderFeatureMetadata, SDFShaderFeatureMetadata};

pub fn upload_system(
    MapContext {
        world,
        style,
        view_state,
        renderer: Renderer { queue, .. },
        ..
    }: &mut MapContext,
) -> SystemResult {
    let Some(Initialized(symbol_buffer_pool)) = world
        .resources
        .query_mut::<&mut Eventually<SymbolBufferPool>>()
    else {
        return Err(SystemError::Dependencies);
    };

    let view_region = view_state.create_view_region(
        view_state.zoom().zoom_level(DEFAULT_TILE_SIZE),
        ViewStatePadding::Loose,
    );

    if let Some(view_region) = &view_region {
        upload_symbol_layer(
            symbol_buffer_pool,
            queue,
            &mut world.tiles,
            style,
            view_region,
        );
    }

    Ok(())
}

// TODO cleanup, duplicated
fn upload_symbol_layer(
    symbol_buffer_pool: &mut SymbolBufferPool,
    queue: &wgpu::Queue,
    tiles: &mut Tiles,
    style: &Style,
    view_region: &ViewRegion,
) {
    // Upload all tessellated layers which are in view
    for coords in view_region.iter() {
        let Some(vector_layers) = tiles.query_mut::<&SymbolLayersDataComponent>(coords) else {
            continue;
        };

        let loaded_layers = symbol_buffer_pool
            .get_loaded_source_layers_at(coords)
            .unwrap_or_default();

        let available_layers = vector_layers
            .layers
            .iter()
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

            let Some(SymbolLayerData {
                coords,
                features,
                buffer,
                ..
            }) = available_layers
                .iter()
                .find(|layer| source_layer.as_str() == layer.source_layer)
            else {
                continue;
            };

            // Assign every feature in the layer the color from the style
            let feature_metadata = iter::repeat(
                SDFShaderFeatureMetadata { opacity: 0.0 }
            ).take(features.last().unwrap().indices.end).collect::<Vec<_>>();

            // FIXME avoid uploading empty indices
            if buffer.buffer.indices.is_empty() {
                continue;
            }

            log::debug!("Allocating geometry at {coords}");
            symbol_buffer_pool.allocate_layer_geometry(
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
