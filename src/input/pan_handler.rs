use super::UpdateState;
use crate::render::camera::{Camera, Perspective};
use crate::render::render_state::RenderState;
use cgmath::{EuclideanSpace, Point3, Vector2, Vector3};
use std::time::Duration;
use winit::event::{ElementState, MouseButton};

pub struct PanHandler {
    window_position: Option<Vector2<f64>>,
    start_window_position: Option<Vector2<f64>>,
    start_camera_position: Option<Vector3<f64>>,
    reference_camera: Option<Camera>,
    is_panning: bool,
}

impl UpdateState for PanHandler {
    fn update_state(&mut self, state: &mut RenderState, _dt: Duration) {
        if !self.is_panning {
            return;
        }

        if let Some(reference_camera) = &self.reference_camera {
            if let (Some(window_position), Some(start_window_position)) =
                (self.window_position, self.start_window_position)
            {
                let perspective = &state.perspective;
                let view_proj = reference_camera.calc_view_proj(perspective);

                let start =
                    reference_camera.project_screen_to_world(&start_window_position, &view_proj);
                let current =
                    reference_camera.project_screen_to_world(&window_position, &view_proj);
                let delta = start - current;

                if self.start_camera_position.is_none() {
                    self.start_camera_position = Some(state.camera.position.to_vec());
                }

                if let Some(start_camera_position) = self.start_camera_position {
                    state.camera.position = Point3::from_vec(
                        start_camera_position + Vector3::new(delta.x, delta.y, 0.0),
                    );
                }
            }
        } else {
            self.reference_camera = Some(state.camera.clone());
        }
    }
}

impl PanHandler {
    pub fn new() -> Self {
        Self {
            window_position: None,
            start_window_position: None,
            start_camera_position: None,
            reference_camera: None,
            is_panning: false,
        }
    }

    pub fn process_touch_start(&mut self) -> bool {
        self.is_panning = true;
        true
    }

    pub fn process_touch_end(&mut self) -> bool {
        self.start_camera_position = None;
        self.start_window_position = None;
        self.window_position = None;
        self.reference_camera = None;
        self.is_panning = false;
        true
    }

    pub fn process_window_position(&mut self, window_position: &Vector2<f64>, touch: bool) -> bool {
        if !self.is_panning && !touch {
            self.start_window_position = Some(*window_position);
            self.window_position = Some(*window_position);
        } else {
            self.window_position = Some(*window_position);
        }

        true
    }
    pub fn process_mouse_key_press(&mut self, key: &MouseButton, state: &ElementState) -> bool {
        if *key != MouseButton::Left {
            return false;
        }

        if !self.is_panning {
            // starting to pan
        }

        if *state == ElementState::Pressed {
            // currently panning or starting to pan
            self.is_panning = true;
        } else {
            // finished panning
            self.start_camera_position = None;
            self.start_window_position = None;
            self.window_position = None;
            self.reference_camera = None;
            self.is_panning = false;
        }
        true
    }
}
