use std::time::Duration;

use cgmath::{EuclideanSpace, Point2, Vector2, Zero};
use maplibre::context::MapContext;
use winit::event::{ElementState, MouseButton};

use super::UpdateState;

#[derive(Default)]
pub struct PanHandler {
    window_position: Option<Vector2<f64>>,
    start_window_position: Option<Vector2<f64>>,
    start_camera_position: Option<Vector2<f64>>,
    is_panning: bool,
}

impl UpdateState for PanHandler {
    fn update_state(&mut self, MapContext { view_state, .. }: &mut MapContext, _dt: Duration) {
        if !self.is_panning {
            return;
        }

        if let (Some(window_position), Some(start_window_position)) =
            (self.window_position, self.start_window_position)
        {
            let view_proj = view_state.view_projection();
            let inverted_view_proj = view_proj.invert();

            let delta = if let (Some(start), Some(current)) = (
                view_state.window_to_world_at_ground(
                    &start_window_position,
                    &inverted_view_proj,
                    false,
                ),
                view_state.window_to_world_at_ground(&window_position, &inverted_view_proj, false),
            ) {
                start - current
            } else {
                Vector2::zero()
            };

            if self.start_camera_position.is_none() {
                self.start_camera_position = Some(view_state.camera().position().to_vec());
            }

            if let Some(start_camera_position) = self.start_camera_position {
                view_state.camera_mut().move_to(Point2::from_vec(
                    start_camera_position + Vector2::new(delta.x, delta.y),
                ));
            }
        }
    }
}

impl PanHandler {
    pub fn process_touch_start(&mut self, window_position: &Vector2<f64>) -> bool {
        self.is_panning = true;
        self.start_window_position = Some(*window_position);
        true
    }

    pub fn process_touch_end(&mut self) -> bool {
        self.start_camera_position = None;
        self.start_window_position = None;
        self.window_position = None;
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

        if *state == ElementState::Pressed {
            // currently panning or starting to pan
            self.is_panning = true;
        } else {
            // finished panning
            self.start_camera_position = None;
            self.start_window_position = None;
            self.window_position = None;
            self.is_panning = false;
        }
        true
    }
}
