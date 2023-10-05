use std::time::Duration;

use cgmath::{Vector3, Zero};
use maplibre::context::MapContext;

use super::UpdateState;

pub struct ShiftHandler {
    camera_translate: Vector3<f64>,

    speed: f64,
    sensitivity: f64,
}

impl UpdateState for ShiftHandler {
    fn update_state(&mut self, MapContext { view_state, .. }: &mut MapContext, dt: Duration) {
        let dt = dt.as_secs_f64() * (1.0 / self.speed);

        let delta = self.camera_translate * dt;
        view_state.camera_mut().move_relative(delta);
        self.camera_translate -= delta;
    }
}

impl ShiftHandler {
    pub fn new(speed: f64, sensitivity: f64) -> Self {
        Self {
            camera_translate: Vector3::zero(),
            speed,
            sensitivity,
        }
    }

    pub fn process_scroll(&mut self, _delta: &winit::event::MouseScrollDelta) {
        /*self.camera_translate.z -= match delta {
            winit::event::MouseScrollDelta::LineDelta(_horizontal, vertical) => *vertical as f64,
            winit::event::MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition {
                y: scroll,
                ..
            }) => *scroll,
        } * self.sensitivity;*/
    }

    pub fn process_key_press(
        &mut self,
        key: winit::event::VirtualKeyCode,
        state: winit::event::ElementState,
    ) -> bool {
        let amount = if state == winit::event::ElementState::Pressed {
            10.0 * self.sensitivity // left, right is the same as panning 10px
        } else {
            0.0
        };
        match key {
            winit::event::VirtualKeyCode::W | winit::event::VirtualKeyCode::Up => {
                self.camera_translate.y -= amount;
                true
            }
            winit::event::VirtualKeyCode::S | winit::event::VirtualKeyCode::Down => {
                self.camera_translate.y += amount;
                true
            }
            winit::event::VirtualKeyCode::A | winit::event::VirtualKeyCode::Left => {
                self.camera_translate.x -= amount;
                true
            }
            winit::event::VirtualKeyCode::D | winit::event::VirtualKeyCode::Right => {
                self.camera_translate.x += amount;
                true
            }
            _ => false,
        }
    }
}
