use cgmath::prelude::*;

use crate::render::shader_ffi::CameraUniform;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[derive(Debug)]
pub struct Camera {
    pub position: cgmath::Point3<f32>,
    pub yaw: cgmath::Rad<f32>,
    pub pitch: cgmath::Rad<f32>,
}

impl Camera {
    pub fn new<
        V: Into<cgmath::Point3<f32>>,
        Y: Into<cgmath::Rad<f32>>,
        P: Into<cgmath::Rad<f32>>,
    >(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }

    pub fn calc_matrix(&self) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::look_to_rh(
            self.position,
            cgmath::Vector3::new(self.yaw.0.cos(), self.pitch.0.sin(), self.yaw.0.sin())
                .normalize(),
            cgmath::Vector3::unit_y(),
        )
    }
    pub fn create_camera_uniform(&self, perspective: &Perspective) -> CameraUniform {
        CameraUniform::new(
            (perspective.calc_matrix() * self.calc_matrix()).into(),
            self.position.to_homogeneous().into(),
        )
    }
}

pub struct Perspective {
    aspect: f32,
    fovy: cgmath::Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Perspective {
    pub fn new<F: Into<cgmath::Rad<f32>>>(
        width: u32,
        height: u32,
        fovy: F,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> cgmath::Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * cgmath::perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}
