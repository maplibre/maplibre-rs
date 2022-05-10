use std::default::Default;

use std::{cmp, iter};

use tracing;

use crate::style::Style;

use crate::coords::{ViewRegion, Zoom};

use crate::io::tile_cache::TileCache;
use crate::io::LayerTessellateMessage;
use crate::platform::MIN_BUFFER_SIZE;
use crate::render::buffer_pool::{BackingBufferDescriptor, BufferPool, IndexEntry};

use crate::render::camera::{Camera, ViewProjection};
use crate::render::options::{INDEX_FORMAT, TILE_VIEW_SIZE};
use crate::render::tile_view_pattern::{TileInView, TileViewPattern};
use crate::tessellation::IndexDataType;
use crate::util::FPSMeter;
use crate::MapWindow;

use super::shaders;
use super::shaders::*;
use super::texture::Texture;

pub struct RenderState2 {
    instance: wgpu::Instance,

    device: wgpu::Device,
    queue: wgpu::Queue,

    fps_meter: FPSMeter,

    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    suspended: bool,

    multisampling_texture: Option<Texture>,

    depth_texture: Texture,
}

impl RenderState {
    pub async fn initialize(
        instance: wgpu::Instance,
        surface: wgpu::Surface,
        surface_config: wgpu::SurfaceConfiguration,
    ) -> Option<Self> {
        let sample_count = 4;

        /*let features = if DEBUG_WIREFRAME {
            wgpu::Features::default() | wgpu::Features::POLYGON_MODE_LINE
        } else {
            wgpu::Features::default()
        };

        surface.configure(&device, &surface_config);*/

        let depth_texture = Texture::create_depth_texture(&device, &surface_config, sample_count);

        let multisampling_texture = if sample_count > 1 {
            Some(Texture::create_multisampling_texture(
                &device,
                &surface_config,
                sample_count,
            ))
        } else {
            None
        };

        Some(Self {
            instance,
            surface,
            device,
            queue,
            surface_config,
            render_pipeline,
            mask_pipeline,
            bind_group,
            multisampling_texture,
            depth_texture,
            sample_count,
            globals_uniform_buffer,
            fps_meter: FPSMeter::new(),
            suspended: false, // Initially rendering is not suspended
        })
    }

    pub fn recreate_surface<W: MapWindow>(&mut self, window: &W) {
        // We only create a new surface if we are currently suspended. On Android (and probably iOS)
        // the surface gets invalid after the app has been suspended.
        if self.suspended {
            let surface = unsafe { self.instance.create_surface(window.inner()) };
            surface.configure(&self.device, &self.surface_config);
            self.surface = surface;
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        // While the app is suspended we can not re-configure a surface
        if self.suspended {
            return;
        }

        self.surface_config.width = width;
        self.surface_config.height = height;

        self.surface.configure(&self.device, &self.surface_config);

        // Re-configure depth buffer
        self.depth_texture =
            Texture::create_depth_texture(&self.device, &self.surface_config, self.sample_count);

        // Re-configure multi-sampling buffer
        self.multisampling_texture = if self.sample_count > 1 {
            Some(Texture::create_multisampling_texture(
                &self.device,
                &self.surface_config,
                self.sample_count,
            ))
        } else {
            None
        };
    }

    #[tracing::instrument(skip_all)]
    pub(crate) fn update_metadata(&mut self) {
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
            .get_tile_cache()
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
    pub fn update_tile_view_pattern(
        &mut self,
        view_region: &ViewRegion,
        view_proj: &ViewProjection,
        zoom: Zoom,
    ) {
        self.tile_view_pattern
            .update_pattern(view_region, &self.buffer_pool, zoom);
        self.tile_view_pattern
            .upload_pattern(&self.queue, view_proj);
    }

    #[tracing::instrument(skip_all)]
    pub fn upload_tile_geometry(
        &mut self,
        view_region: &ViewRegion,
        style: &Style,
        tile_cache: &TileCache,
    ) {
        // Upload all tessellated layers which are in view
        for world_coords in view_region.iter() {
            let loaded_layers = self
                .buffer_pool
                .get_loaded_layers_at(&world_coords)
                .unwrap_or_default();
            if let Some(available_layers) = tile_cache
                .iter_tessellated_layers_at(&world_coords)
                .map(|layers| {
                    layers
                        .filter(|result| !loaded_layers.contains(&result.layer_name()))
                        .collect::<Vec<_>>()
                })
            {
                for style_layer in &style.layers {
                    let source_layer = style_layer.source_layer.as_ref().unwrap();

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
                            LayerTessellateMessage::UnavailableLayer { coords: _, .. } => {
                                /*self.buffer_pool.mark_layer_unavailable(*coords);*/
                            }
                            LayerTessellateMessage::TessellatedLayer {
                                coords,
                                feature_indices,
                                layer_data,
                                buffer,
                                ..
                            } => {
                                let allocate_feature_metadata = tracing::span!(
                                    tracing::Level::TRACE,
                                    "allocate_feature_metadata"
                                );

                                let guard = allocate_feature_metadata.enter();
                                let feature_metadata = layer_data
                                    .features
                                    .iter()
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
                                self.buffer_pool.allocate_layer_geometry(
                                    &self.queue,
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

    #[tracing::instrument(skip_all)]
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let render_setup_span = tracing::span!(tracing::Level::TRACE, "render prepare");
        let _guard = render_setup_span.enter();

        let frame = self.surface.get_current_texture()?;
        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Encoder"),
            });

        drop(_guard);

        {
            let _span_ = tracing::span!(tracing::Level::TRACE, "render pass").entered();
            {
                /* let color_attachment =
                    if let Some(multisampling_target) = &self.multisampling_texture {
                        wgpu::RenderPassColorAttachment {
                            view: &multisampling_target.view,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                                store: true,
                            },
                            resolve_target: Some(&frame_view),
                        }
                    } else {
                        wgpu::RenderPassColorAttachment {
                            view: &frame_view,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                                store: true,
                            },
                            resolve_target: None,
                        }
                    };

                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[color_attachment],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_texture.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(0.0),
                            store: true,
                        }),
                        stencil_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(0),
                            store: true,
                        }),
                    }),
                });*/

                pass.set_bind_group(0, &self.bind_group, &[]);

                /*{
                    let index = self.buffer_pool.index();

                    for TileInView { shape, fallback } in self.tile_view_pattern.iter() {
                        let coords = shape.coords;
                        tracing::trace!("Drawing tile at {coords}");

                        let shape_to_render = fallback.as_ref().unwrap_or(shape);

                        let reference = self
                            .tile_view_pattern
                            .stencil_reference_value(&shape_to_render.coords)
                            as u32;

                        // Draw mask
                        /*{
                            tracing::trace!("Drawing mask {}", &coords);

                            pass.set_pipeline(&self.mask_pipeline);
                            pass.set_stencil_reference(reference);
                            pass.set_vertex_buffer(
                                0,
                                self.tile_view_pattern
                                    .buffer()
                                    .slice(shape.buffer_range.clone()),
                            );
                            pass.draw(0..6, 0..1);
                        }*/

                        if let Some(entries) = index.get_layers(&shape_to_render.coords) {
                            let mut layers_to_render: Vec<&IndexEntry> = Vec::from_iter(entries);
                            layers_to_render.sort_by_key(|entry| entry.style_layer.index);

                            for entry in layers_to_render {
                                // Draw tile
                                /* {
                                    tracing::trace!(
                                        "Drawing layer {:?} at {}",
                                        entry.style_layer.source_layer,
                                        &entry.coords
                                    );

                                    pass.set_pipeline(&self.render_pipeline);
                                    pass.set_stencil_reference(reference);
                                    pass.set_index_buffer(
                                        self.buffer_pool
                                            .indices()
                                            .slice(entry.indices_buffer_range()),
                                        INDEX_FORMAT,
                                    );
                                    pass.set_vertex_buffer(
                                        0,
                                        self.buffer_pool
                                            .vertices()
                                            .slice(entry.vertices_buffer_range()),
                                    );
                                    pass.set_vertex_buffer(
                                        1,
                                        self.tile_view_pattern
                                            .buffer()
                                            .slice(shape_to_render.buffer_range.clone()),
                                    );
                                    pass.set_vertex_buffer(
                                        2,
                                        self.buffer_pool
                                            .metadata()
                                            .slice(entry.layer_metadata_buffer_range()),
                                    );
                                    pass.set_vertex_buffer(
                                        3,
                                        self.buffer_pool
                                            .feature_metadata()
                                            .slice(entry.feature_metadata_buffer_range()),
                                    );
                                    pass.draw_indexed(entry.indices_range(), 0, 0..1);
                                }*/
                            }
                        } else {
                            tracing::trace!("No layers found at {}", &shape_to_render.coords);
                        }
                    }
                }*/
            }
        }

        {
            let _span = tracing::span!(tracing::Level::TRACE, "render finish").entered();
            tracing::trace!("Finished drawing");

            self.queue.submit(Some(encoder.finish()));
            tracing::trace!("Submitted queue");

            frame.present();
            tracing::trace!("Presented frame");
        }

        self.fps_meter.update_and_print();
        Ok(())
    }

    pub fn suspend(&mut self) {
        self.suspended = true;
    }

    pub fn resume(&mut self) {
        self.suspended = false;
    }
}
