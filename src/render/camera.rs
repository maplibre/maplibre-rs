use cgmath::prelude::*;
use cgmath::{Matrix4, Vector2, Vector3, Vector4};

use crate::render::shaders::ShaderCamera;

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

#[derive(Debug, Clone)]
pub struct Camera {
    pub translation: cgmath::Matrix4<f64>,
    pub position: cgmath::Point3<f64>,
    pub yaw: cgmath::Rad<f64>,
    pub pitch: cgmath::Rad<f64>,

    pub width: f64,
    pub height: f64,
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
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            position: position.into(),
            translation: Matrix4::identity(),
            yaw: yaw.into(),
            pitch: pitch.into(),
            width: width as f64,
            height: height as f64,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width as f64;
        self.height = height as f64;
    }

    fn calc_matrix(&self) -> cgmath::Matrix4<f64> {
        cgmath::Matrix4::look_to_rh(
            self.position,
            cgmath::Vector3::new(self.yaw.0.cos(), self.pitch.0.sin(), self.yaw.0.sin())
                .normalize(),
            cgmath::Vector3::unit_y(),
        ) * self.translation
    }

    pub fn calc_view_proj(&self, perspective: &Perspective) -> Matrix4<f64> {
        FLIP_Y * perspective.calc_matrix() * self.calc_matrix()
    }

    pub fn create_camera_uniform(&self, perspective: &Perspective) -> ShaderCamera {
        let view_proj = self.calc_view_proj(perspective);
        ShaderCamera::new(
            view_proj.cast::<f32>().unwrap().into(),
            self.position.to_homogeneous().cast::<f32>().unwrap().into(),
        )
    }

    fn dx_matrix(width: f64, height: f64) -> Matrix4<f64> {
        // Adapted from: https://docs.microsoft.com/en-us/windows/win32/direct3d9/viewports-and-clipping#viewport-rectangle
        let min_depth = 0.0;
        let max_depth = 1.0;
        let x = 0.0;
        let y = 0.0;
        let ox = x + width / 2.0;
        let oy = y + height / 2.0;
        let oz = min_depth;
        let px = width as f64;
        let py = height as f64;
        let pz = max_depth - min_depth;
        Matrix4::from_cols(
            Vector4::new(width as f64 / 2.0, 0.0, 0.0, 0.0),
            Vector4::new(0.0, -height as f64 / 2.0, 0.0, 0.0),
            Vector4::new(0.0, 0.0, pz, 0.0),
            Vector4::new(ox, oy, oz, 1.0),
        )
    }

    fn clip_to_window(clip: &Vector4<f64>, width: f64, height: f64) -> Vector3<f64> {
        // Adopted from: https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkViewport.html
        // and https://matthewwellings.com/blog/the-new-vulkan-coordinate-system/
        #[rustfmt::skip]
            let ndc = Vector4::new(
            clip.x / clip.w,
            clip.y / clip.w,
            clip.z / clip.w,
            clip.w / clip.w
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
        let xd = ndc.x;
        let yd = ndc.y;
        let zd = ndc.z;
        Vector3::new(px / 2.0 * xd + ox, py / 2.0 * yd + oy, pz * zd + oz)
    }

    // https://docs.microsoft.com/en-us/windows/win32/dxtecharts/the-direct3d-transformation-pipeline
    fn clip_to_window_matrix(clip: &Vector4<f64>, width: f64, height: f64) -> Vector4<f64> {
        let w = clip.w;
        let z = clip.z;
        println!("z in clip space: {z}");
        println!("w in clip space: {w}");

        #[rustfmt::skip]
            let ndc = Vector4::new(
            clip.x / clip.w,
            clip.y / clip.w,
            clip.z / clip.w,
            clip.w / clip.w
        );

        let window = Self::dx_matrix(width, height) * ndc;
        window
    }

    // https://docs.microsoft.com/en-us/windows/win32/dxtecharts/the-direct3d-transformation-pipeline
    fn window_to_clip(
        window: &Vector3<f64>,
        origin_clip_space: &Vector4<f64>,
        width: f64,
        height: f64,
    ) -> Vector4<f64> {
        let z = window.z;
        println!("z in window space: {z}");
        #[rustfmt::skip]
            let fixed_window = Vector4::new(
            window.x,
            window.y,
            window.z,
            1.0
        );

        let ndc = Self::dx_matrix(width, height).invert().unwrap() * fixed_window;

        let w = origin_clip_space.w;

        #[rustfmt::skip]
            let clip = Vector4::new(
            ndc.x * w,
            ndc.y * w,
            ndc.z * w,
            w,
        );

        clip
    }

    fn window_to_world(
        window: &Vector3<f64>,
        view_proj: &Matrix4<f64>,
        width: f64,
        height: f64,
    ) -> Vector3<f64> {
        #[rustfmt::skip]
            let fixed_window = Vector4::new(
            window.x,
            window.y,
            window.z,
            1.0
        );

        let ndc = Self::dx_matrix(width, height).invert().unwrap() * fixed_window;
        let unprojected = view_proj.invert().unwrap() * ndc;
        let world = Vector3::new(
            unprojected.x / unprojected.w,
            unprojected.y / unprojected.w,
            unprojected.z / unprojected.w,
        );
        world
    }

    fn window_to_world_nalgebra(
        window: &Vector3<f64>,
        view_proj: &Matrix4<f64>,
        width: f64,
        height: f64,
    ) -> Vector3<f64> {
        let pt = Vector4::new(
            2.0 * (window.x - 0.0) / width - 1.0,
            2.0 * (window.y - 0.0) / height - 1.0,
            window.z,
            1.0,
        );

        /*        // opengl
                let pt = Vector4::new(
                    2.0 * (window.x - 0.0) / width - 1.0,
                    2.0 * (window.y - 0.0) / height - 1.0,
                    2.0 * window.z - 1.0,
                    1.0,
                );
        */
        let unprojected_nalgebra = view_proj.invert().unwrap() * pt;
        let world = Vector3::new(
            unprojected_nalgebra.x / unprojected_nalgebra.w,
            unprojected_nalgebra.y / unprojected_nalgebra.w,
            unprojected_nalgebra.z / unprojected_nalgebra.w,
        );
        world
    }

    pub fn project_screen_to_world(
        &self,
        window: &Vector2<f64>,
        view_proj: &Matrix4<f64>,
    ) -> Vector3<f64> {
        /*        let origin_clip_space = (view_proj * Vector4::new(0.0, 0.0, 0.0, 1.0));

        let origin_window_space =
            Self::clip_to_window_matrix(&origin_clip_space, self.width, self.height);
        let reverse_clip = Self::window_to_clip(
            &Vector3::new(window.x, window.y, origin_window_space.z),
            &origin_clip_space,
            self.width,
            self.height,
        );*/

        let near_world = Camera::window_to_world(
            &Vector3::new(window.x, window.y, 0.0),
            &view_proj,
            self.width,
            self.height,
        );

        let far_world = Camera::window_to_world(
            &Vector3::new(window.x, window.y, 1.0),
            &view_proj,
            self.width,
            self.height,
        );

        // for z = 0 in world coordinates
        let u = -near_world.z / (far_world.z - near_world.z);
        println!("u: {:?}", u);

        /*let vec = (near_world - far_world).normalize();
        let znear = 0.1;
        near_world + znear * vec*/

        near_world + u * (far_world - near_world)
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

    pub fn calc_matrix(&self) -> cgmath::Matrix4<f64> {
        OPENGL_TO_WGPU_MATRIX * cgmath::perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

#[cfg(test)]
mod tests {
    use cgmath::{AbsDiffEq, ElementWise, Matrix4, SquareMatrix, Vector2, Vector3, Vector4};

    use super::{Camera, Perspective};

    #[test]
    fn test() {
        let width = 1920.0;
        let height = 1080.0;
        let camera = Camera::new(
            (0.0, 5.0, 5000.0),
            cgmath::Deg(-90.0),
            cgmath::Deg(45.0),
            width as u32,
            height as u32,
        );
        // 4732.561319582916
        let perspective = Perspective::new(
            width as u32,
            height as u32,
            cgmath::Deg(45.0),
            0.1,
            100000.0,
        );
        let view_proj: Matrix4<f64> = camera.calc_view_proj(&perspective);

        let world_pos: Vector4<f64> = Vector4::new(0.0, 0.0, 0.0, 1.0);
        let clip = view_proj * world_pos;

        let origin_clip_space = view_proj * Vector4::new(0.0, 0.0, 0.0, 1.0);
        println!("origin w in clip space: {:?}", origin_clip_space.w);

        println!("world_pos: {:?}", world_pos);
        println!("clip: {:?}", clip);
        println!("world_pos: {:?}", view_proj.invert().unwrap() * clip);

        println!("window: {:?}", Camera::clip_to_window(&clip, width, height));
        let window = Camera::clip_to_window_matrix(&clip, width, height);
        println!("window (matrix): {:?}", window);

        let origin_window_space = Camera::clip_to_window_matrix(&origin_clip_space, width, height);
        let reverse_clip = Camera::window_to_clip(
            &Vector3::new(window.x, window.y, origin_window_space.z),
            &origin_clip_space,
            width,
            height,
        );
        let reverse_world = view_proj.invert().unwrap() * reverse_clip;

        println!("r clip: {:?}", reverse_clip);
        println!("r world: {:?}", reverse_world);

        // --------- nalgebra

        let scale = 1.0 / origin_clip_space.w;

        let origin_window_space_nalgebra = Vector3::new(
            0.0 + (width * (origin_clip_space.x * scale + 1.0) * 0.5),
            0.0 + (height * (origin_clip_space.y * scale + 1.0) * 0.5),
            origin_clip_space.z * scale,
        );
        println!("r origin (nalgebra): {:?}", origin_window_space_nalgebra);
        println!(
            "r world (nalgebra): {:?}",
            Camera::window_to_world_nalgebra(&window.truncate(), &view_proj, width, height)
        );
        // --------

        // pdf trick
        let near_world = Camera::window_to_world_nalgebra(
            &Vector3::new(window.x, window.y, 0.0),
            &view_proj,
            width,
            height,
        );

        let far_world = Camera::window_to_world_nalgebra(
            &Vector3::new(window.x, window.y, 1.0),
            &view_proj,
            width,
            height,
        );

        // for z = 0 in world coordinates
        let u = -near_world.z / (far_world.z - near_world.z);
        println!("u: {:?}", u);
        let unprojected = near_world + u * (far_world - near_world);
        println!("unprojected: {:?}", unprojected);
        //assert!(Vector3::new(world_pos.x, world_pos.y, world_pos.z).abs_diff_eq(&unprojected, 0.05));
        //.------

        // ---- test for unproject

        let window = Vector2::new(960.0, 631.0); // 0, 4096: passt nicht
                                                 //let window = Vector2::new(962.0, 1.0); // 0, 300: passt nicht
                                                 //let window = Vector2::new(960.0, 540.0); // 0, 0 passt
        let near_world = Camera::window_to_world(
            &Vector3::new(window.x, window.y, 0.0),
            &view_proj,
            width,
            height,
        );

        let far_world = Camera::window_to_world(
            &Vector3::new(window.x, window.y, 1.0),
            &view_proj,
            width,
            height,
        );

        // for z = 0 in world coordinates
        let u = -near_world.z / (far_world.z - near_world.z);
        println!("u: {:?}", u);
        let unprojected = near_world + u * (far_world - near_world);
        println!("unprojected: {:?}", unprojected);
        // ----

        //assert!(reverse_world.abs_diff_eq(&world_pos, 0.05))
    }
}
