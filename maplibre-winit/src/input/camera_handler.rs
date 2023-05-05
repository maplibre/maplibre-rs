use std::time::Duration;

use cgmath::{Deg, Zero};
use maplibre::context::MapContext;

use super::UpdateState;

pub struct CameraHandler {
    delta_pitch: Deg<f64>,
    delta_roll: Deg<f64>,
    delta_yaw: Deg<f64>,

    speed: f64,
    sensitivity: f64,
}

impl UpdateState for CameraHandler {
    fn update_state(&mut self, MapContext { view_state, .. }: &mut MapContext, dt: Duration) {
        let dt = dt.as_secs_f64() * (1.0 / self.speed);

        let delta_pitch = self.delta_pitch * dt;
        view_state.camera_mut().pitch(delta_pitch);
        self.delta_pitch -= delta_pitch;

        let delta_roll = self.delta_roll * dt;
        view_state.camera_mut().roll(delta_roll);
        self.delta_roll -= delta_roll;

        let delta_yaw = self.delta_yaw * dt;
        view_state.camera_mut().yaw(delta_yaw);
        self.delta_yaw -= delta_yaw;
    }
}

impl CameraHandler {
    pub fn new(speed: f64, sensitivity: f64) -> Self {
        Self {
            delta_pitch: Deg::zero(),
            delta_roll: Deg::zero(),
            delta_yaw: Deg::zero(),
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
            winit::event::VirtualKeyCode::T => {
                self.delta_yaw -= amount;
                true
            }
            winit::event::VirtualKeyCode::G => {
                self.delta_yaw += amount;
                true
            }
            winit::event::VirtualKeyCode::Z => {
                self.delta_roll -= amount;
                true
            }
            winit::event::VirtualKeyCode::H => {
                self.delta_roll += amount;
                true
            }
            _ => false,
        }
    }
}
