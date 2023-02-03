//! Uploads data to the GPU which is needed for rendering.

use std::iter;

use crate::{
    context::MapContext,
    coords::ViewRegion,
    ecs::world::World,
    io::tile_repository::{StoredLayer, TileRepository},
    render::{
        camera::ViewProjection,
        eventually::Eventually::Initialized,
        shaders::{ShaderCamera, ShaderFeatureStyle, ShaderGlobals, ShaderLayerMetadata, Vec4f32},
        RenderState, Renderer,
    },
    schedule::Stage,
    style::Style,
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
            renderer:
                Renderer {
                    device,
                    queue,
                    state,
                    ..
                },
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
            self.upload_tesselated_layer(state, device, queue, tile_repository, style, view_region);
            self.upload_raster_layer(state, device, queue, tile_repository, style, view_region);
            self.upload_tile_view_pattern(state, queue, &view_proj);
            //self.update_metadata(state, tile_repository, queue);
        }
    }
}

impl UploadStage {
    #[tracing::instrument(skip_all)]
    pub(crate) fn update_metadata(
        &self,
        RenderState { buffer_pool, .. }: &mut RenderState,
        tile_repository: &TileRepository,
        queue: &wgpu::Queue,
    ) {
        let Initialized(buffer_pool) = buffer_pool else { return; };

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
    pub fn upload_tile_view_pattern(
        &self,
        RenderState {
            tile_view_pattern, ..
        }: &mut RenderState,
        queue: &wgpu::Queue,
        view_proj: &ViewProjection,
    ) {
        let Initialized(tile_view_pattern) = tile_view_pattern else { return; };
        tile_view_pattern.upload_pattern(queue, view_proj);
    }

    #[tracing::instrument(skip_all)]
    pub fn upload_tesselated_layer(
        &self,
        RenderState { buffer_pool, .. }: &mut RenderState,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        tile_repository: &TileRepository,
        style: &Style,
        view_region: &ViewRegion,
    ) {
        let Initialized(buffer_pool) = buffer_pool else { return; };

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
    pub fn upload_raster_layer(
        &self,
        RenderState {
            raster_resources, ..
        }: &mut RenderState,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        tile_repository: &TileRepository,
        style: &Style,
        view_region: &ViewRegion,
    ) {
        let Initialized(raster_resources) = raster_resources else { return; };

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
}
