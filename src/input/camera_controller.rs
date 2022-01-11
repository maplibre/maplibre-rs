use std::f32::consts::FRAC_PI_2;

use crate::render::camera;
use cgmath::{EuclideanSpace, InnerSpace, Matrix4, SquareMatrix, Vector2, Vector3, Vector4};
use log::info;

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Debug)]
pub struct CameraController {
    translate_x: f64,
    translate_y: f64,
    direct_translate_x: f64,
    direct_translate_y: f64,

    zoom: f64,

    speed: f64,
    sensitivity: f64,
}

impl CameraController {
    pub fn new(speed: f64, sensitivity: f64) -> Self {
        Self {
            translate_x: 0.0,
            translate_y: 0.0,
            direct_translate_x: 0.0,
            direct_translate_y: 0.0,
            zoom: 0.0,
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
                self.translate_y += amount;
                true
            }
            winit::event::VirtualKeyCode::S | winit::event::VirtualKeyCode::Down => {
                self.translate_y -= amount;
                true
            }
            winit::event::VirtualKeyCode::A | winit::event::VirtualKeyCode::Left => {
                self.translate_x -= amount;
                true
            }
            winit::event::VirtualKeyCode::D | winit::event::VirtualKeyCode::Right => {
                self.translate_x += amount;
                true
            }
            winit::event::VirtualKeyCode::Space => {
                self.translate_y = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(
        &mut self,
        start_cam_x: f64,
        start_cam_y: f64,
        mouse_dx: f64,
        mouse_dy: f64,
        width: f64,
        height: f64,
        camera: &mut camera::Camera,
        view_proj: &Matrix4<f64>,
    ) {
        info!("mouse_dx {} mouse_dy {}", mouse_dx, mouse_dy);

        let origin = Vector2::new(0.0, 0.0);
        let screen = Vector2::new(mouse_dx, mouse_dy);
        let camera_pos = &camera.position.to_vec();
        let world = Self::screen_to_world(&origin, width, height, camera_pos, &view_proj)
            - Self::screen_to_world(&screen, width, height, camera_pos, &view_proj);

        info!("world {:?}", world);

        //self.direct_translate_x -= world.x;
        //self.direct_translate_y += world.y;
        camera.position.x = start_cam_x - world.x;
        camera.position.y = start_cam_y + world.y;
    }

    fn screen_to_world(
        screen: &Vector2<f64>,
        width: f64,
        height: f64,
        camera_pos: &Vector3<f64>,
        view_proj: &Matrix4<f64>,
    ) -> Vector4<f64> {
        let min_depth = 0.0;
        let max_depth = 1.0;

        let x = 0.0;
        let y = 0.0;
        let ox = x + width / 2.0;
        let oy = y + height / 2.0;
        let oz = min_depth;
        let pz = max_depth - min_depth;

        // Adapted from: https://docs.microsoft.com/en-us/windows/win32/direct3d9/viewports-and-clipping#viewport-rectangle
        let direct_x = Matrix4::from_cols(
            Vector4::new(width as f64 / 2.0, 0.0, 0.0, 0.0),
            Vector4::new(0.0, height as f64 / 2.0, 0.0, 0.0),
            Vector4::new(0.0, 0.0, pz, 0.0),
            Vector4::new(ox, oy, oz, 1.0),
        );

        let screen_hom = Vector4::new(screen.x, screen.y, 1.0, 1.0) * camera_pos.z;
        let result = direct_x.invert().unwrap() * screen_hom;
        let world_pos = view_proj.invert().unwrap() * result;
        world_pos
    }

    pub fn process_touch(&mut self, touch_dx: f64, touch_dy: f64) {
        self.translate_x += touch_dx as f64 * self.sensitivity;
        self.translate_y += touch_dy as f64 * self.sensitivity;
    }

    pub fn process_scroll(&mut self, delta: &winit::event::MouseScrollDelta) {
        self.zoom = -match delta {
            // I'm assuming a line is about 100 pixels
            winit::event::MouseScrollDelta::LineDelta(_, scroll) => *scroll as f64 * 100.0,
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
        camera.position.x += self.direct_translate_x;
        camera.position.y += self.direct_translate_y;
        self.direct_translate_x = 0.0;
        self.direct_translate_y = 0.0;

        let dt = dt.as_secs_f64() * self.speed;

        let dy = self.translate_y * dt;
        camera.position.y += dy;
        let dx = self.translate_x * dt;
        camera.position.x += dx;

        // Move in/out (aka. "zoom")
        // Note: this isn't an actual zoom. The camera's position
        // changes when zooming. I've added this to make it easier
        // to get closer to an object you want to focus on.
        let dz = self.zoom * dt;
        camera.position.z += dz;

        self.zoom -= dz;
        self.translate_x -= dx;
        self.translate_y -= dy;
    }
}
