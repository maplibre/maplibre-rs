use std::time::Duration;

use cgmath::{Deg, Zero};
use maplibre::context::MapContext;

use super::UpdateState;

pub struct TiltHandler {
    delta_pitch: Deg<f64>,

    speed: f64,
    sensitivity: f64,
}

impl UpdateState for TiltHandler {
    fn update_state(&mut self, MapContext { view_state, .. }: &mut MapContext, dt: Duration) {
        let dt = dt.as_secs_f64() * (1.0 / self.speed);

        let delta = self.delta_pitch * dt;
        view_state.camera_mut().tilt(delta);
        self.delta_pitch -= delta;
    }
}

impl TiltHandler {
    pub fn new(speed: f64, sensitivity: f64) -> Self {
        Self {
            delta_pitch: Deg::zero(),
            speed,
            sensitivity,
        }
    }

    pub fn process_key_press(
        &mut self,
        key: winit::event::VirtualKeyCode,
        state: winit::event::ElementState,
    ) -> bool {
        let amount = if state == winit::event::ElementState::Pressed {
            Deg(0.1 * self.sensitivity) // left, right is the same as panning 1 degree
        } else {
            Deg::zero()
        };
        match key {
            winit::event::VirtualKeyCode::R => {
                self.delta_pitch -= amount;
                true
            }
            winit::event::VirtualKeyCode::F => {
                self.delta_pitch += amount;
                true
            }
            _ => false,
        }
    }
}
