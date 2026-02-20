use wgpu::util::DeviceExt;

use crate::{
    context::MapContext,
    render::{
        render_phase::{DrawState, LayerItem, RenderPhase},
        shaders::BackgroundLayerMetadata,
    },
    style::layer::LayerPaint,
    tcs::system::{SystemError, SystemResult},
};

pub struct BackgroundBuffers {
    pub metadata_buffer: wgpu::Buffer,
}

use super::render_commands::DrawBackground;

pub fn queue_system(
    MapContext {
        world,
        style,
        renderer,
        ..
    }: &mut MapContext,
) -> SystemResult {
    let Some(mut layer_item_phase) = world.resources.get_mut::<RenderPhase<LayerItem>>() else {
        return Err(SystemError::Dependencies);
    };

    let mut metadatas = Vec::new();

    // Note: Background layer is uniquely not tied to any tiles.
    // We just iterate through the style layers and issue a single quad draw for each background layer.
    for layer in &style.layers {
        if layer.type_ == "background" {
            let c = match &layer.paint {
                Some(LayerPaint::Background(paint)) => paint
                    .background_color
                    .as_ref()
                    .map(|c| c.to_array())
                    .unwrap_or([0.0, 0.0, 0.0, 1.0]),
                _ => [0.0, 0.0, 0.0, 1.0],
            };
            let color = [c[0] as f32, c[1] as f32, c[2] as f32, c[3] as f32];
            let z_index = layer.index as f32;
            metadatas.push(BackgroundLayerMetadata { color, z_index });

            layer_item_phase.add(LayerItem {
                draw_function: Box::new(DrawState::<LayerItem, DrawBackground>::new())
                    as Box<dyn crate::render::render_phase::Draw<LayerItem>>,
                index: layer.index,
                style_layer: layer.id.clone(),
                source_shape: crate::render::tile_view_pattern::TileShape::default(),

                // We provide a dummy tile for background.
                tile: crate::tcs::tiles::Tile {
                    coords: crate::coords::WorldTileCoords::default(),
                },
            });
        }
    }

    if !metadatas.is_empty() {
        let buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Background Metadata Buffer"),
                contents: bytemuck::cast_slice(&metadatas),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
        world.resources.insert(BackgroundBuffers {
            metadata_buffer: buffer,
        });
    }

    Ok(())
}
