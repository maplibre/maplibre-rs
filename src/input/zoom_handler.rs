use super::UpdateState;

use crate::io::tile_cache::TileCache;
use crate::render::render_state::RenderState;
use cgmath::num_traits::Pow;
use cgmath::{Vector2, Vector3};
use std::time::Duration;

pub struct ZoomHandler {
    window_position: Option<Vector2<f64>>,
    zoom_delta: f64,
    sensitivity: f64,
}

impl UpdateState for ZoomHandler {
    fn update_state(&mut self, state: &mut RenderState, _tile_cache: &TileCache, _dt: Duration) {
        if self.zoom_delta != 0.0 {
            if let Some(window_position) = self.window_position {
                let current_zoom = state.zoom;
                let next_zoom = current_zoom + self.zoom_delta;

                state.zoom = next_zoom;
                self.zoom_delta = 0.0;
                println!("zoom: {}", state.zoom);

                let perspective = &state.perspective;
                let view_proj = state.camera.calc_view_proj(perspective);
                let inverted_view_proj = view_proj.invert();

                if let Some(cursor_position) = state
                    .camera
                    .window_to_world_at_ground(&window_position, &inverted_view_proj)
                {
                    let scale = 2.0.pow(next_zoom - current_zoom);

                    let delta = Vector3::new(
                        cursor_position.x * scale,
                        cursor_position.y * scale,
                        cursor_position.z,
                    ) - cursor_position;

                    state.camera.position += delta;
                }
            }
        }
    }
}

impl ZoomHandler {
    pub fn new(sensitivity: f64) -> Self {
        Self {
            window_position: None,
            zoom_delta: 0.0,
            sensitivity,
        }
    }

    pub fn process_window_position(
        &mut self,
        window_position: &Vector2<f64>,
        _touch: bool,
    ) -> bool {
        self.window_position = Some(*window_position);
        true
    }

    pub fn process_scroll(&mut self, delta: &winit::event::MouseScrollDelta) {
        self.zoom_delta += match delta {
            winit::event::MouseScrollDelta::LineDelta(_horizontal, vertical) => *vertical as f64,
            winit::event::MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition {
                y: scroll,
                ..
            }) => *scroll / 100.0,
        } * self.sensitivity;
    }

    pub fn process_key_press(
        &mut self,
        key: winit::event::VirtualKeyCode,
        state: winit::event::ElementState,
    ) -> bool {
        let amount = if state == winit::event::ElementState::Pressed {
            0.1
        } else {
            0.0
        };

        match key {
            winit::event::VirtualKeyCode::Plus | winit::event::VirtualKeyCode::I => {
                self.zoom_delta += amount;
                true
            }
            winit::event::VirtualKeyCode::Minus | winit::event::VirtualKeyCode::K => {
                self.zoom_delta -= amount;
                true
            }
            _ => false,
        }
    }
}
