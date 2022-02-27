use super::UpdateState;
use crate::coords::{WorldCoords, EXTENT};
use crate::render::camera::Camera;
use crate::render::render_state::RenderState;
use cgmath::num_traits::Pow;
use cgmath::{ulps_eq, EuclideanSpace, Matrix4, Point3, Vector2, Vector3, Vector4, Zero};
use std::time::Duration;

pub struct ZoomHandler {
    window_position: Option<Vector2<f64>>,
    translate_delta: Vector3<f64>,
    zooming: bool,
    zoom_delta: f64,

    speed: f64,
    sensitivity: f64,
}

impl UpdateState for ZoomHandler {
    fn update_state(&mut self, state: &mut RenderState, dt: Duration) {
        if self.zoom_delta != 0.0 {
            if let Some(window_position) = self.window_position {
                let current_zoom = state.zoom;
                let next_zoom = current_zoom + self.zoom_delta;

                state.zoom = next_zoom;
                self.zoom_delta = 0.0;

                let perspective = &state.perspective;
                let view_proj = state.camera.calc_view_proj(perspective);

                if let Some(window_position_world) = state
                    .camera
                    .window_to_world_z0(&window_position, &view_proj)
                {
                    let scale = 2.0.pow(next_zoom - current_zoom);

                    let delta = Vector3::new(
                        window_position_world.x * scale,
                        window_position_world.y * scale,
                        window_position_world.z,
                    ) - window_position_world;

                    state.camera.position += delta;
                }
            }
        }
    }
}

impl ZoomHandler {
    pub fn new(speed: f64, sensitivity: f64) -> Self {
        Self {
            window_position: None,
            translate_delta: Vector3::zero(),
            zooming: false,
            zoom_delta: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_window_position(&mut self, window_position: &Vector2<f64>, touch: bool) -> bool {
        self.window_position = Some(*window_position);
        true
    }

    pub fn process_scroll(&mut self, delta: &winit::event::MouseScrollDelta) {
        self.zoom_delta += match delta {
            winit::event::MouseScrollDelta::LineDelta(_horizontal, vertical) => *vertical as f64,
            winit::event::MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition {
                y: scroll,
                ..
            }) => *scroll,
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
