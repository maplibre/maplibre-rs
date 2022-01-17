//! Handles the user input which is dispatched by the main event loop.

use std::time::Duration;

use cgmath::Vector2;
use winit::event::{
    DeviceEvent, ElementState, KeyboardInput, MouseButton, TouchPhase, WindowEvent,
};

use crate::input::camera_controller::CameraController;
use crate::render::camera::Camera;
use crate::render::render_state::RenderState;

mod camera_controller;

pub struct InputHandler {
    camera_controller: CameraController,

    mouse_position: Option<Vector2<f64>>,
    last_mouse_position: Option<Vector2<f64>>,
    initial_camera: Option<Camera>,
    initial_camera_position: Option<cgmath::Point3<f64>>,
    mouse_pressed: bool,
}

impl InputHandler {
    pub fn new() -> Self {
        let camera_controller = CameraController::new(5.0, 100.0);
        Self {
            initial_camera_position: None,
            mouse_position: None,
            last_mouse_position: None,
            initial_camera: None,
            mouse_pressed: false,
            camera_controller,
        }
    }

    pub fn device_input(&mut self, event: &DeviceEvent) -> bool {
        match event {
            _ => false,
        }
    }

    fn pan_camera(&mut self, position: Vector2<f64>, render_state: &mut RenderState) {
        if let (Some(last_mouse_position), Some(initial_camera_position), Some(initial_camera)) = (
            self.last_mouse_position,
            self.initial_camera_position,
            self.initial_camera.as_ref(),
        ) {
            self.camera_controller.pan_camera(
                initial_camera_position,
                last_mouse_position,
                position,
                initial_camera,
                &render_state.camera,
                &render_state.perspective,
            );
        } else {
            self.last_mouse_position = Some(position);
            self.initial_camera_position = Some(render_state.camera.position);
            self.initial_camera = Some(render_state.camera.clone());
        }
    }

    pub fn window_input(&mut self, event: &WindowEvent, render_state: &mut RenderState) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                if self.mouse_pressed {
                    let mouse_position: (f64, f64) = position.to_owned().into();
                    self.pan_camera(Vector2::from(mouse_position), render_state);
                    self.mouse_position = Some(Vector2::from(mouse_position));
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
                _ => self.camera_controller.process_keyboard(*key, *state),
            },
            WindowEvent::Touch(touch) => {
                let touch_position: (f64, f64) = touch.location.to_owned().into();
                match touch.phase {
                    TouchPhase::Started => {
                        self.last_mouse_position = Some(Vector2::from(touch_position));
                        self.initial_camera_position = Some(render_state.camera.position);
                    }
                    TouchPhase::Moved | TouchPhase::Ended => {
                        self.pan_camera(Vector2::from(touch_position), render_state);
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
                    /*if let (
                        Some(last_mouse_position),
                        Some(initial_camera_position),
                        Some(initial_camera),
                        Some(mouse_position),
                    ) = (
                        self.last_mouse_position,
                        self.initial_camera_position,
                        self.initial_camera.as_ref(),
                        self.mouse_position,
                    ) {
                        self.camera_controller.pan_camera(
                            initial_camera_position,
                            last_mouse_position,
                            mouse_position,
                            initial_camera,
                            &render_state.camera,
                            &render_state.perspective,
                        );
                    }*/

                    self.last_mouse_position = None;
                    self.initial_camera_position = None;
                    self.initial_camera = None;
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
