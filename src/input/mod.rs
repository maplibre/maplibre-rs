use cgmath::Vector3;
use std::time::Duration;
use winit::event::{
    DeviceEvent, ElementState, KeyboardInput, MouseButton, TouchPhase, WindowEvent,
};
use winit::window::Window;

use crate::input::camera_controller::CameraController;
use crate::render::camera::Camera;
use crate::render::state::State;

mod camera_controller;

pub struct InputHandler {
    camera_controller: CameraController,

    last_touch: Option<(f64, f64)>,
    start_camera_pos: Option<cgmath::Point3<f64>>,
    mouse_pressed: bool,
    target_stroke_width: f32,
}

impl InputHandler {
    pub fn new() -> Self {
        let camera_controller = CameraController::new(5.0, 100.0);
        Self {
            target_stroke_width: 1.0,
            start_camera_pos: None,
            last_touch: None,
            mouse_pressed: false,
            camera_controller,
        }
    }

    pub fn device_input(
        &mut self,
        event: &DeviceEvent,
        state: &mut State,
        window: &Window,
    ) -> bool {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                /*                if self.mouse_pressed {
                    let view_proj = state.camera.calc_view_proj(&state.perspective);
                    self.camera_controller.process_mouse(
                        delta.0 / window.scale_factor(),
                        delta.1 / window.scale_factor(),
                        state.size.width as f64,
                        state.size.height as f64,
                        &mut state.camera,
                        &view_proj,
                    );
                }*/
                true
            }
            _ => false,
        }
    }

    pub fn window_input(
        &mut self,
        event: &WindowEvent,
        state: &mut State,
        window: &Window,
    ) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                if self.mouse_pressed {
                    if let Some(start) = self.last_touch {
                        let delta_x = start.0 - position.x;
                        let delta_y = start.1 - position.y;
                        let view_proj = state.camera.calc_view_proj(&state.perspective);
                        self.camera_controller.process_mouse(
                            self.start_camera_pos.unwrap().x,
                            self.start_camera_pos.unwrap().y,
                            delta_x,
                            delta_y,
                            state.size.width as f64,
                            state.size.height as f64,
                            &mut state.camera,
                            &view_proj,
                        );
                    } else {
                        self.last_touch = Some((position.x, position.y));
                        self.start_camera_pos = Some(state.camera.position);
                    }
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
                match touch.phase {
                    TouchPhase::Started => {
                        self.last_touch = Some((touch.location.x, touch.location.y));
                        self.start_camera_pos = Some(state.camera.position);
                    }
                    TouchPhase::Ended => {
                        self.last_touch = None;
                        self.start_camera_pos = None;
                    }
                    TouchPhase::Moved => {
                        if let Some(start) = self.last_touch {
                            let delta_x = start.0 - touch.location.x;
                            let delta_y = start.1 - touch.location.y;
                            let view_proj = state.camera.calc_view_proj(&state.perspective);
                            self.camera_controller.process_mouse(
                                self.start_camera_pos.unwrap().x,
                                self.start_camera_pos.unwrap().y,
                                delta_x,
                                delta_y,
                                state.size.width as f64,
                                state.size.height as f64,
                                &mut state.camera,
                                &view_proj,
                            );
                        }
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
                    self.last_touch = None;
                    self.start_camera_pos = None;
                }
                true
            }
            _ => false,
        }
    }

    pub fn update_state(&mut self, state: &mut State, dt: Duration) {
        let scene = &mut state.scene;
        self.camera_controller.update_camera(&mut state.camera, dt);

        // Animate the stroke_width to match target_stroke_width
        scene.stroke_width =
            scene.stroke_width + (self.target_stroke_width - scene.stroke_width) / 5.0;

        // Animate the strokes of primitive
        /*        scene.cpu_primitives[0 as usize].width = scene.stroke_width;*/
        /*
        scene.cpu_primitives[STROKE_PRIM_ID as usize].color = [
                    (time_secs * 0.8 - 1.6).sin() * 0.1 + 0.1,
                    (time_secs * 0.5 - 1.6).sin() * 0.1 + 0.1,
                    (time_secs - 1.6).sin() * 0.1 + 0.1,
                    1.0,
        ];
        */
    }
}
