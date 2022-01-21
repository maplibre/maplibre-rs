//! Handles the user input which is dispatched by the main event loop.

use std::time::Duration;

use cgmath::{Vector2};
use winit::event::{DeviceEvent, KeyboardInput, TouchPhase, WindowEvent};

use crate::input::pan_handler::PanHandler;
use crate::input::pinch_handler::PinchHandler;
use crate::input::shift_handler::ShiftHandler;
use crate::input::tilt_handler::TiltHandler;
use crate::render::render_state::RenderState;

mod pan_handler;
mod pinch_handler;
mod shift_handler;
mod tilt_handler;

pub struct InputController {
    pinch_handler: PinchHandler,
    pan_handler: PanHandler,
    tilt_handler: TiltHandler,
    shift_handler: ShiftHandler,
}

impl InputController {
    /// Creates a new input controller.
    ///
    /// # Arguments
    ///
    /// * `speed`: How fast animation should go. Default is 1.0. 2.0 is a speedup of 2.
    /// * `sensitivity`: How much impact an action has. Default is 10px for pressing the forward
    /// key for example.
    ///
    /// returns: InputController
    ///
    pub fn new(speed: f64, sensitivity: f64) -> Self {
        Self {
            pinch_handler: PinchHandler::new(),
            pan_handler: PanHandler::new(),
            tilt_handler: TiltHandler::new(speed, sensitivity),
            shift_handler: ShiftHandler::new(speed, sensitivity),
        }
    }

    pub fn device_input(&mut self, _event: &DeviceEvent) -> bool {
        false
    }

    pub fn window_input(&mut self, event: &WindowEvent, _render_state: &RenderState) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let position: (f64, f64) = position.to_owned().into();
                self.pan_handler
                    .process_window_position(&Vector2::from(position), false)
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(key),
                        ..
                    },
                ..
            } => {
                self.shift_handler.process_key_press(*key, *state);
                self.tilt_handler.process_key_press(*key, *state);
                true
            },
            WindowEvent::Touch(touch) => match touch.phase {
                TouchPhase::Started => self.pan_handler.process_touch_start(),
                TouchPhase::Ended => self.pan_handler.process_touch_end(),
                TouchPhase::Moved => {
                    let position: (f64, f64) = touch.location.to_owned().into();
                    self.pan_handler
                        .process_window_position(&Vector2::from(position), true)
                }
                TouchPhase::Cancelled => false,
            },
            WindowEvent::MouseWheel { delta, .. } => {
                self.shift_handler.process_scroll(delta);
                true
            }
            WindowEvent::MouseInput { button, state, .. } => {
                self.pan_handler.process_mouse_key_press(button, state)
            }
            _ => false,
        }
    }
}

pub trait UpdateState {
    fn update_state(&mut self, state: &mut RenderState, dt: Duration);
}

impl UpdateState for InputController {
    fn update_state(&mut self, state: &mut RenderState, dt: Duration) {
        self.pan_handler.update_state(state, dt);
        self.pinch_handler.update_state(state, dt);
        self.tilt_handler.update_state(state, dt);
        self.shift_handler.update_state(state, dt);
    }
}
