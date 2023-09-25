use std::time::Duration;

use cgmath::{Deg, MetricSpace, Rad, Vector2};
use maplibre::context::MapContext;
use winit::event::{ElementState, MouseButton};

use super::UpdateState;

pub struct CameraHandler {
    window_position: Option<Vector2<f64>>,
    start_window_position: Option<Vector2<f64>>,
    is_active: bool,
    is_middle: bool,

    start_delta_pitch: Option<Rad<f64>>,
    start_delta_roll: Option<Rad<f64>>,
    start_delta_yaw: Option<Rad<f64>>,

    sensitivity: f64,
}

impl UpdateState for CameraHandler {
    fn update_state(&mut self, MapContext { view_state, .. }: &mut MapContext, dt: Duration) {
        if !self.is_active {
            return;
        }

        if let (Some(window_position), Some(start_window_position)) =
            (self.window_position, self.start_window_position)
        {
            let camera = view_state.camera_mut();

            if self.is_middle {
                let delta: Rad<_> = (Deg(0.001 * self.sensitivity)
                    * start_window_position.distance(window_position))
                .into();

                let previous = *self.start_delta_roll.get_or_insert(camera.get_roll());
                camera.set_roll(previous + delta);
            } else {
                let delta: Rad<_> = (Deg(0.001 * self.sensitivity)
                    * (start_window_position.x - window_position.x))
                    .into();
                let previous = *self.start_delta_yaw.get_or_insert(camera.get_yaw());
                camera.set_yaw(previous + delta);

                let delta: Rad<_> = (Deg(0.001 * self.sensitivity)
                    * (start_window_position.y - window_position.y))
                    .into();
                let previous = *self.start_delta_pitch.get_or_insert(camera.get_pitch());
                camera.set_pitch(previous + delta);
            }
        }
    }
}

impl CameraHandler {
    pub fn new(sensitivity: f64) -> Self {
        Self {
            window_position: None,
            start_window_position: None,
            is_active: false,
            is_middle: false,
            start_delta_pitch: None,
            start_delta_roll: None,
            start_delta_yaw: None,
            sensitivity,
        }
    }

    pub fn process_window_position(&mut self, window_position: &Vector2<f64>, touch: bool) -> bool {
        if !self.is_active && !touch {
            self.start_window_position = Some(*window_position);
            self.window_position = Some(*window_position);
        } else {
            self.window_position = Some(*window_position);
        }

        true
    }

    pub fn process_mouse_key_press(&mut self, key: &MouseButton, state: &ElementState) -> bool {
        if *state == ElementState::Pressed {
            // currently panning or starting to pan
            match *key {
                MouseButton::Right => {
                    self.is_active = true;
                }
                MouseButton::Middle => {
                    self.is_active = true;
                    self.is_middle = true;
                }
                _ => return false,
            }
        } else {
            // finished panning
            self.is_active = false;
            self.is_middle = false;
            self.start_window_position = None;
            self.window_position = None;
            self.start_delta_yaw = None;
            self.start_delta_pitch = None;
            self.start_delta_roll = None;
        }
        true
    }
}
