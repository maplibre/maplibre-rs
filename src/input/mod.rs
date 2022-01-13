//! Handles the user input which is dispatched by the main event loop.

use std::time::Duration;

use cgmath::Vector2;
use winit::event::{
    DeviceEvent, ElementState, KeyboardInput, MouseButton, TouchPhase, WindowEvent,
};

use crate::input::camera_controller::CameraController;
use crate::render::render_state::RenderState;

mod camera_controller;

pub struct InputHandler {
    camera_controller: CameraController,

    last_mouse_position: Option<Vector2<f64>>,
    initial_camera_position: Option<cgmath::Point3<f64>>,
    mouse_pressed: bool,
    target_stroke_width: f32,
}

impl InputHandler {
    pub fn new() -> Self {
        let camera_controller = CameraController::new(5.0, 100.0);
        Self {
            target_stroke_width: 1.0,
            initial_camera_position: None,
            last_mouse_position: None,
            mouse_pressed: false,
            camera_controller,
        }
    }

    pub fn device_input(&mut self, event: &DeviceEvent) -> bool {
        match event {
            _ => false,
        }
    }

    fn process_mouse_delta(&mut self, position: Vector2<f64>, state: &mut RenderState) {
        if let (Some(last_mouse_position), Some(initial_camera_position)) =
            (self.last_mouse_position, self.initial_camera_position)
        {
            let delta = last_mouse_position - position;
            self.camera_controller.process_mouse(
                initial_camera_position,
                delta,
                &state.camera,
                &state.perspective,
            );
        } else {
            self.last_mouse_position = Some(position);
            self.initial_camera_position = Some(state.camera.position);
        }
    }

    pub fn window_input(&mut self, event: &WindowEvent, state: &mut RenderState) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                if self.mouse_pressed {
                    let mouse_position: (f64, f64) = position.to_owned().into();
                    self.process_mouse_delta(Vector2::from(mouse_position), state);
                }
                true
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(key),
                        ..
                    },
                ..
            } => match key {
                winit::event::VirtualKeyCode::Z => {
                    self.target_stroke_width *= 1.2;
                    true
                }
                winit::event::VirtualKeyCode::H => {
                    self.target_stroke_width *= 0.8;
                    true
                }
                _ => self.camera_controller.process_keyboard(*key, *state),
            },
            WindowEvent::Touch(touch) => {
                let touch_position: (f64, f64) = touch.location.to_owned().into();
                match touch.phase {
                    TouchPhase::Started => {
                        self.last_mouse_position = Some(Vector2::from(touch_position));
                        self.initial_camera_position = Some(state.camera.position);
                    }
                    TouchPhase::Moved | TouchPhase::Ended => {
                        self.process_mouse_delta(Vector2::from(touch_position), state);
                    }
                    TouchPhase::Cancelled => {}
                }

                true
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left, // Left Mouse Button
                state,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;

                if !self.mouse_pressed {
                    self.last_mouse_position = None;
                    self.initial_camera_position = None;
                }
                true
            }
            _ => false,
        }
    }

    pub fn update_state(&mut self, state: &mut RenderState, dt: Duration) {
        self.camera_controller.update_camera(&mut state.camera, dt);
    }
}
