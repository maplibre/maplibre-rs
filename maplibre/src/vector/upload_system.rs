//! Uploads data to the GPU which is needed for rendering.

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
    style::{layer::LayerPaint, Style},
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

    let zoom = view_state.zoom().level();

    if let Some(view_region) = &view_region {
        upload_tessellated_layer(
            buffer_pool,
            device,
            queue,
            &mut world.tiles,
            style,
            view_region,
            zoom,
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
    zoom: f32,
) {
    // Upload all tessellated layers which are in view
    for coords in view_region.iter() {
        let Some(vector_layers) = tiles.query_mut::<&VectorLayerBucketComponent>(coords) else {
            continue;
        };

        let loaded_layers = buffer_pool
            .get_loaded_style_layers_at(coords)
            .unwrap_or_default();

        let available_layers = vector_layers
            .layers
            .iter()
            .flat_map(|data| match data {
                VectorLayerBucket::AvailableLayer(data) => Some(data),
                VectorLayerBucket::Missing(_) => None,
            })
            .filter(|data| !loaded_layers.contains(data.style_layer_id.as_str()))
            .collect::<Vec<_>>();

        for style_layer in &style.layers {
            let layer_id = &style_layer.id;
            // GeoJSON sources have no source_layer; fall back to the layer id as a
            // virtual source-layer name (matches the name set in process_geojson_features).
            let source_layer = match style_layer.source_layer.as_ref() {
                Some(layer) => layer.as_str(),
                None => {
                    log::trace!("style layer {layer_id} has no source_layer, using id as virtual source layer");
                    style_layer.id.as_str()
                }
            };

            let Some(AvailableVectorLayerBucket {
                coords,
                feature_indices,
                feature_colors,
                buffer,
                ..
            }) = available_layers
                .iter()
                .find(|layer| style_layer.id.as_str() == layer.style_layer_id.as_str())
            else {
                continue;
            };

            let color: Option<Vec4f32> = style_layer
                .paint
                .as_ref()
                .and_then(|paint| paint.get_color())
                .map(|color| color.into());

            // Assign every feature in the layer the color from the style if no parsed feature_color exist.
            let fallback_color = color.unwrap_or([0.0, 0.0, 0.0, 1.0]);

            let mut feature_metadata =
                Vec::with_capacity(feature_indices.iter().sum::<u32>() as usize);
            for (idx, &count) in feature_indices.iter().enumerate() {
                let current_color = feature_colors.get(idx).copied().unwrap_or(fallback_color);
                for _ in 0..count {
                    feature_metadata.push(FillShaderFeatureMetadata {
                        color: current_color,
                    });
                }
            }

            // FIXME avoid uploading empty indices
            if buffer.buffer.indices.is_empty() {
                continue;
            }

            // Extract line-width from style paint (default 1.0px)
            let line_width = match &style_layer.paint {
                Some(LayerPaint::Line(paint)) => paint
                    .line_width
                    .as_ref()
                    .map(|w| w.evaluate_at_zoom(zoom))
                    .unwrap_or(1.0),
                _ => 1.0,
            };

            log::debug!("Allocating geometry at {coords}");
            buffer_pool.allocate_layer_geometry(
                queue,
                *coords,
                style_layer.clone(),
                buffer,
                ShaderLayerMetadata {
                    z_index: style_layer.index as f32,
                    line_width,
                },
                &feature_metadata,
            );
        }
    }
}
