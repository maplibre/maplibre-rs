//! Uploads data to the GPU which is needed for rendering.

use std::iter;

use crate::{
    context::MapContext,
    coords::ViewRegion,
    render::{
        eventually::{Eventually, Eventually::Initialized},
        shaders::{ShaderLayerMetadata, Vec4f32},
        tile_view_pattern::DEFAULT_TILE_SIZE,
        Renderer,
    },
    sdf::{SymbolBufferPool, SymbolLayerData, SymbolLayersDataComponent},
    style::Style,
    tcs::tiles::Tiles,
};

pub fn upload_system(
    MapContext {
        world,
        style,
        view_state,
        renderer: Renderer { queue, .. },
        ..
    }: &mut MapContext,
) {
    let Some(Initialized(symbol_buffer_pool)) = world
        .resources
        .query_mut::<&mut Eventually<SymbolBufferPool>>()
    else {
        return;
    };

    let view_region =
        view_state.create_view_region(view_state.zoom().zoom_level(DEFAULT_TILE_SIZE));

    if let Some(view_region) = &view_region {
        upload_symbol_layer(
            symbol_buffer_pool,
            queue,
            &mut world.tiles,
            style,
            view_region,
        );
    }
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
            let source_layer = style_layer.source_layer.as_ref().unwrap(); // TODO: Unwrap

            let Some(SymbolLayerData {
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

            // Assign every feature in the layer the color from the style
            let feature_metadata = (0..feature_indices.len()) // FIXME: Iterate over actual features
                .enumerate()
                .flat_map(|(i, _feature)| {
                    iter::repeat(crate::render::shaders::ShaderFeatureStyle {
                        color: Vec4f32::default(),
                    })
                    .take(feature_indices[i] as usize)
                })
                .collect::<Vec<_>>();

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
                ShaderLayerMetadata::new(style_layer.index as f32),
                &feature_metadata,
            );
        }
    }
}