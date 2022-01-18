use super::UpdateState;
use crate::render::render_state::RenderState;
use cgmath::{Deg, Rad, Zero};
use std::time::Duration;

pub struct TiltHandler {
    delta_pitch: Deg<f64>,

    speed: f64,
    sensitivity: f64,
}

impl UpdateState for TiltHandler {
    fn update_state(&mut self, state: &mut RenderState, dt: Duration) {
        let dt = dt.as_secs_f64() * (1.0 / self.speed);

        let delta = self.delta_pitch * dt;
        state.camera.pitch += Rad::from(delta);
        let x: Deg<f64> = state.camera.pitch.into();
        println!("{:?}", x);
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
