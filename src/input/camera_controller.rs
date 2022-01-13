use cgmath::{EuclideanSpace, Point3, Vector2, Vector3, Zero};

use crate::render::camera;

#[derive(Debug)]
pub struct CameraController {
    camera_position: Option<Vector3<f64>>,
    camera_translate: Vector3<f64>,

    speed: f64,
    sensitivity: f64,
}

impl CameraController {
    pub fn new(speed: f64, sensitivity: f64) -> Self {
        Self {
            camera_position: None,
            camera_translate: Vector3::zero(),
            speed,
            sensitivity,
        }
    }

    pub fn process_keyboard(
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
                self.camera_translate.y += amount;
                true
            }
            winit::event::VirtualKeyCode::S | winit::event::VirtualKeyCode::Down => {
                self.camera_translate.y -= amount;
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

    pub fn process_mouse(
        &mut self,
        initial_camera_position: cgmath::Point3<f64>,
        delta: Vector2<f64>,
        camera: &camera::Camera,
        perspective: &camera::Perspective,
    ) {
        let view_proj = camera.calc_view_proj(perspective);
        let world = camera.project_screen_to_world(&Vector2::new(0.0, 0.0), &view_proj)
            - camera.project_screen_to_world(&delta, &view_proj);

        self.camera_position =
            Some(initial_camera_position.to_vec() + Vector3::new(-world.x, world.y, 0.0))
    }

    pub fn process_scroll(&mut self, delta: &winit::event::MouseScrollDelta) {
        self.camera_translate.z -= match delta {
            winit::event::MouseScrollDelta::LineDelta(_, scroll) => *scroll as f64 * 50.0,
            winit::event::MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition {
                y: scroll,
                ..
            }) => *scroll,
        } * self.sensitivity;
    }

    pub fn update_camera(
        &mut self,
        camera: &mut crate::render::camera::Camera,
        dt: std::time::Duration,
    ) {
        let dt = dt.as_secs_f64() * self.speed;

        if let Some(position) = self.camera_position {
            camera.position = Point3::from_vec(position);
            self.camera_position = None;
        }

        // Move in/out (aka. "zoom")
        // Note: this isn't an actual zoom. The camera's position
        // changes when zooming. I've added this to make it easier
        // to get closer to an object you want to focus on.
        let delta = self.camera_translate * dt;
        camera.position += delta;

        self.camera_translate -= delta;
    }
}
