//! Uploads data to the GPU which is needed for rendering.

use std::iter;

use crate::{
    context::MapContext,
    coords::ViewRegion,
    io::tile_repository::{StoredLayer, TileRepository},
    render::{
        camera::ViewProjection,
        eventually::Eventually::Initialized,
        shaders::{ShaderCamera, ShaderFeatureStyle, ShaderGlobals, ShaderLayerMetadata, Vec4f32},
        RenderState, Renderer,
    },
    schedule::Stage,
    style::Style,
    world::World,
};

#[derive(Default)]
pub struct UploadStage;

impl Stage for UploadStage {
    #[tracing::instrument(name = "UploadStage", skip_all)]
    fn run(
        &mut self,
        MapContext {
            world:
                World {
                    tile_repository,
                    view_state,
                    ..
                },
            style,
            renderer: Renderer { queue, state, .. },
            ..
        }: &mut MapContext,
    ) {
        let view_proj = view_state.view_projection();

        if let Initialized(globals_bind_group) = &state.globals_bind_group {
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
            self.upload_tile_geometry(state, queue, tile_repository, style, view_region);
            self.upload_tile_view_pattern(state, queue, &view_proj);
            self.update_metadata();
        }
    }
}

impl UploadStage {
    #[tracing::instrument(skip_all)]
    pub(crate) fn update_metadata(&self) {
        /*let animated_one = 0.5
        * (1.0
            + ((SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64()
                * 10.0)
                .sin()));*/

        // Factor which determines how much we need to adjust the width of lines for example.
        // If zoom == z -> zoom_factor == 1

        /*  for entries in self.buffer_pool.index().iter() {
        for entry in entries {
            let world_coords = entry.coords;*/

        // TODO: Update features
        /*let source_layer = entry.style_layer.source_layer.as_ref().unwrap();

        if let Some(result) = scheduler
            .get_tile_repository()
            .iter_tessellated_layers_at(&world_coords)
            .unwrap()
            .find(|layer| source_layer.as_str() == layer.layer_name())
        {
            let color: Option<Vec4f32> = entry
                .style_layer
                .paint
                .as_ref()
                .and_then(|paint| paint.get_color())
                .map(|mut color| {
                    color.color.b = animated_one as f32;
                    color.into()
                });

            match result {
                LayerTessellateResult::UnavailableLayer { .. } => {}
                LayerTessellateResult::TessellatedLayer {
                    layer_data,
                    feature_indices,
                    ..
                } => {

                    let feature_metadata = layer_data
                        .features()
                        .iter()
                        .enumerate()
                        .flat_map(|(i, _feature)| {
                            iter::repeat(ShaderFeatureStyle {
                                color: color.unwrap(),
                            })
                            .take(feature_indices[i] as usize)
                        })
                        .collect::<Vec<_>>();

                    self.buffer_pool.update_feature_metadata(
                        &self.queue,
                        entry,
                        &feature_metadata,
                    );
                }
            }
        }*/
        /*            }
        }*/
    }

    #[tracing::instrument(skip_all)]
    pub fn upload_tile_view_pattern(
        &self,
        RenderState {
            tile_view_pattern, ..
        }: &mut RenderState,
        queue: &wgpu::Queue,
        view_proj: &ViewProjection,
    ) {
        if let Initialized(tile_view_pattern) = tile_view_pattern {
            tile_view_pattern.upload_pattern(queue, view_proj);
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn upload_tile_geometry(
        &self,
        RenderState { buffer_pool, .. }: &mut RenderState,
        queue: &wgpu::Queue,
        tile_repository: &TileRepository,
        style: &Style,
        view_region: &ViewRegion,
    ) {
        if let Initialized(buffer_pool) = buffer_pool {
            // Upload all tessellated layers which are in view
            for world_coords in view_region.iter() {
                let loaded_layers = buffer_pool
                    .get_loaded_layers_at(&world_coords)
                    .unwrap_or_default();
                if let Some(available_layers) =
                    tile_repository.iter_layers_at(&world_coords).map(|layers| {
                        layers
                            .filter(|result| !loaded_layers.contains(&result.layer_name()))
                            .collect::<Vec<_>>()
                    })
                {
                    for style_layer in &style.layers {
                        let source_layer = style_layer.source_layer.as_ref().unwrap(); // TODO: Remove unwrap

                        if let Some(message) = available_layers
                            .iter()
                            .find(|layer| source_layer.as_str() == layer.layer_name())
                        {
                            let color: Option<Vec4f32> = style_layer
                                .paint
                                .as_ref()
                                .and_then(|paint| paint.get_color())
                                .map(|color| color.into());

                            match message {
                                StoredLayer::UnavailableLayer { coords: _, .. } => {
                                    /*self.buffer_pool.mark_layer_unavailable(*coords);*/
                                }
                                StoredLayer::TessellatedLayer {
                                    coords,
                                    feature_indices,
                                    buffer,
                                    ..
                                } => {
                                    let allocate_feature_metadata = tracing::span!(
                                        tracing::Level::TRACE,
                                        "allocate_feature_metadata"
                                    );

                                    let guard = allocate_feature_metadata.enter();
                                    let feature_metadata =
                                        (0..feature_indices.len()) // FIXME: Iterate over actual featrues
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
            }
        }
    }
}
