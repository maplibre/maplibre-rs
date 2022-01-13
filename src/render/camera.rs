use cgmath::prelude::*;
use cgmath::Matrix4;

use crate::render::shader_ffi::CameraUniform;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f64> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[rustfmt::skip]
pub const FLIP_Y: cgmath::Matrix4<f64> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0, 
    0.0, -1.0, 0.0, 0.0, 
    0.0, 0.0, 1.0, 0.0, 
    0.0, 0.0, 0.0, 1.0,
);

#[derive(Debug)]
pub struct Camera {
    pub position: cgmath::Point3<f64>,
    pub yaw: cgmath::Rad<f64>,
    pub pitch: cgmath::Rad<f64>,
}

impl Camera {
    pub fn new<
        V: Into<cgmath::Point3<f64>>,
        Y: Into<cgmath::Rad<f64>>,
        P: Into<cgmath::Rad<f64>>,
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

    fn calc_matrix(&self) -> cgmath::Matrix4<f64> {
        cgmath::Matrix4::look_to_rh(
            self.position,
            cgmath::Vector3::new(self.yaw.0.cos(), self.pitch.0.sin(), self.yaw.0.sin())
                .normalize(),
            cgmath::Vector3::unit_y(),
        )
    }

    pub fn calc_view_proj(&self, perspective: &Perspective) -> Matrix4<f64> {
        FLIP_Y * perspective.calc_matrix() * self.calc_matrix()
    }

    pub fn create_camera_uniform(&self, perspective: &Perspective) -> CameraUniform {
        let view_proj = self.calc_view_proj(perspective);
        CameraUniform::new(
            view_proj.cast::<f32>().unwrap().into(),
            self.position.to_homogeneous().cast::<f32>().unwrap().into(),
        )
    }
}

pub struct Perspective {
    aspect: f64,
    fovy: cgmath::Rad<f64>,
    znear: f64,
    zfar: f64,
}

impl Perspective {
    pub fn new<F: Into<cgmath::Rad<f64>>>(
        width: u32,
        height: u32,
        fovy: F,
        znear: f64,
        zfar: f64,
    ) -> Self {
        Self {
            aspect: width as f64 / height as f64,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f64 / height as f64;
    }

    fn calc_matrix(&self) -> cgmath::Matrix4<f64> {
        OPENGL_TO_WGPU_MATRIX * cgmath::perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

#[cfg(test)]
mod tests {
    use cgmath::{ElementWise, Matrix4, SquareMatrix, Vector3, Vector4};

    use super::{Camera, Perspective};

    #[test]
    fn test() {
        let camera = Camera::new((0.0, 5.0, 5000.0), cgmath::Deg(-90.0), cgmath::Deg(-0.0));
        let width = 1920;
        let height = 1080;
        let perspective = Perspective::new(width, height, cgmath::Deg(45.0), 0.1, 100000.0);
        let projection = perspective;

        let world_pos: Vector4<f64> = Vector4::new(2000.0, 2000.0, 0.0, 1.0);
        println!("world_pos: {:?}", world_pos);
        let view_proj: Matrix4<f64> = (projection.calc_matrix() * camera.calc_matrix())
            .cast()
            .unwrap();

        let result = view_proj * world_pos;
        println!("result: {:?}", result);

        // Adopted from: https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkViewport.html
        // and https://matthewwellings.com/blog/the-new-vulkan-coordinate-system/
        let result_ndc = Vector4::new(
            result.x / result.w,
            result.y / result.w,
            result.z / result.w,
            result.w,
        );

        let min_depth = 0.0;
        let max_depth = 1.0;

        let x = 0.0;
        let y = 0.0;
        let ox = x + width as f64 / 2.0;
        let oy = y + height as f64 / 2.0;
        let oz = min_depth;
        let px = width as f64;
        let py = height as f64;
        let pz = max_depth - min_depth;
        let xd = result_ndc.x;
        let yd = result_ndc.y;
        let zd = result_ndc.z;
        let screen = Vector3::new(px / 2.0 * xd + ox, py / 2.0 * yd + oy, pz * zd + oz);
        println!("screen: {:?}", screen);

        // Adapted from: https://docs.microsoft.com/en-us/windows/win32/direct3d9/viewports-and-clipping#viewport-rectangle
        let direct_x = Matrix4::from_cols(
            Vector4::new(width as f64 / 2.0, 0.0, 0.0, 0.0),
            Vector4::new(0.0, height as f64 / 2.0, 0.0, 0.0),
            Vector4::new(0.0, 0.0, pz, 0.0),
            Vector4::new(ox, oy, oz, 1.0),
        );
        let screen_hom = direct_x * result;
        println!("screen_hom: {:?}", screen_hom);
        let screen = Vector3::new(
            screen_hom.x / screen_hom.w,
            screen_hom.y / screen_hom.w,
            screen_hom.z / screen_hom.w,
        );
        println!("screen: {:?}", screen);

        let screen_hom2 = Vector4::new(screen.x, screen.y, 1.0, 1.0) * 5000.0;
        println!("screen_hom2: {:?}", screen_hom2);
        let result = direct_x.invert().unwrap() * screen_hom;
        println!("result screen_hom: {:?}", result);
        let result = direct_x.invert().unwrap() * screen_hom2;
        println!("result screen_hom2: {:?}", result);
        let world_pos = view_proj.invert().unwrap() * result;
        println!("world_pos: {:?}", world_pos);
    }
}
