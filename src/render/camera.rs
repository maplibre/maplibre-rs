use cgmath::prelude::*;
use cgmath::{Matrix4, Point2, Point3, Vector2, Vector3, Vector4};

use crate::render::shaders::ShaderCamera;
use crate::util::math::{Aabb2, Aabb3, Plane};

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
            cgmath::Vector3::new(self.yaw.cos(), self.pitch.sin(), self.yaw.sin()).normalize(),
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

    fn clip_to_window_transform(width: f64, height: f64) -> Matrix4<f64> {
        // Adapted from: https://docs.microsoft.com/en-us/windows/win32/direct3d9/viewports-and-clipping#viewport-rectangle
        let min_depth = 0.0;
        let max_depth = 1.0;
        let x = 0.0;
        let y = 0.0;
        let ox = x + width / 2.0;
        let oy = y + height / 2.0;
        let oz = min_depth;
        let pz = max_depth - min_depth;
        Matrix4::from_cols(
            Vector4::new(width as f64 / 2.0, 0.0, 0.0, 0.0),
            Vector4::new(0.0, -height as f64 / 2.0, 0.0, 0.0),
            Vector4::new(0.0, 0.0, pz, 0.0),
            Vector4::new(ox, oy, oz, 1.0),
        )
    }

    // Adopted from: https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkViewport.html
    // and https://matthewwellings.com/blog/the-new-vulkan-coordinate-system/
    fn clip_to_window(clip: &Vector4<f64>, width: f64, height: f64) -> Vector3<f64> {
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
    fn clip_to_window_via_transform(&self, clip: &Vector4<f64>) -> Vector4<f64> {
        #[rustfmt::skip]
        let ndc = Vector4::new(
            clip.x / clip.w,
            clip.y / clip.w,
            clip.z / clip.w,
            clip.w / clip.w
        );

        
        Self::clip_to_window_transform(self.width, self.height) * ndc
    }

    /// Order of transformations reversed: https://computergraphics.stackexchange.com/questions/6087/screen-space-coordinates-to-eye-space-conversion/6093
    /// `w` is lost.
    ///
    /// OpenGL explanation: https://www.khronos.org/opengl/wiki/Compute_eye_space_from_window_space#From_window_to_ndc
    fn window_to_world(&self, window: &Vector3<f64>, view_proj: &Matrix4<f64>) -> Vector3<f64> {
        #[rustfmt::skip]
            let fixed_window = Vector4::new(
            window.x,
            window.y,
            window.z,
            1.0
        );

        let ndc = Self::clip_to_window_transform(self.width, self.height)
            .invert()
            .unwrap()
            * fixed_window;
        let unprojected = view_proj.invert().unwrap() * ndc;
        
        Vector3::new(
            unprojected.x / unprojected.w,
            unprojected.y / unprojected.w,
            unprojected.z / unprojected.w,
        )
    }

    /// Alternative implementation to `window_to_world`
    ///
    /// https://docs.rs/nalgebra-glm/latest/src/nalgebra_glm/ext/matrix_projection.rs.html#164-181
    fn window_to_world_nalgebra(
        window: &Vector3<f64>,
        view_proj: &Matrix4<f64>,
        width: f64,
        height: f64,
    ) -> Vector3<f64> {
        let pt = Vector4::new(
            2.0 * (window.x - 0.0) / width - 1.0,
            2.0 * (height - window.y - 0.0) / height - 1.0,
            window.z,
            1.0,
        );
        let unprojected_nalgebra = view_proj.invert().unwrap() * pt;
        
        Vector3::new(
            unprojected_nalgebra.x / unprojected_nalgebra.w,
            unprojected_nalgebra.y / unprojected_nalgebra.w,
            unprojected_nalgebra.z / unprojected_nalgebra.w,
        )
    }

    /// Idea comes from: https://dondi.lmu.build/share/cg/unproject-explained.pdf
    pub fn window_to_world_z0(
        &self,
        window: &Vector2<f64>,
        view_proj: &Matrix4<f64>,
    ) -> Vector3<f64> {
        let near_world = self.window_to_world(&Vector3::new(window.x, window.y, 0.0), view_proj);

        let far_world = self.window_to_world(&Vector3::new(window.x, window.y, 1.0), view_proj);

        // for z = 0 in world coordinates
        let u = -near_world.z / (far_world.z - near_world.z);
        if !(0.0..=1.0).contains(&u) {
            panic!("interpolation factor is out of bounds")
        }
        near_world + u * (far_world - near_world)
    }

    pub fn view_bounding_box2(&self, perspective: &Perspective) -> Option<Aabb2<f64>> {
        let view_proj = self.calc_view_proj(perspective);

        let vec = vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(self.width, 0.0),
            Vector2::new(self.width, self.height),
            Vector2::new(0.0, self.height),
        ]
        .iter()
        .map(|point| self.window_to_world_z0(point, &view_proj))
        .collect::<Vec<_>>();

        let min_x = vec
            .iter()
            .map(|point| point.x)
            .min_by(|a, b| a.partial_cmp(b).unwrap())?;
        let min_y = vec
            .iter()
            .map(|point| point.y)
            .min_by(|a, b| a.partial_cmp(b).unwrap())?;
        let max_x = vec
            .iter()
            .map(|point| point.x)
            .max_by(|a, b| a.partial_cmp(b).unwrap())?;
        let max_y = vec
            .iter()
            .map(|point| point.y)
            .max_by(|a, b| a.partial_cmp(b).unwrap())?;
        Some(Aabb2::new(
            Point2::new(min_x, min_y),
            Point2::new(max_x, max_y),
        ))
    }

    pub fn view_bounding_box(&self, perspective: &Perspective) -> Option<Aabb2<f64>> {
        let view_proj = self.calc_view_proj(perspective);
        let a = view_proj * Vector4::new(0.0, 0.0, 0.0, 1.0);
        let b = view_proj * Vector4::new(1.0, 0.0, 0.0, 1.0);
        let c = view_proj * Vector4::new(1.0, 1.0, 0.0, 1.0);

        let a = self.clip_to_window_via_transform(&a);
        let a_ndc = a.truncate();
        let b_ndc = self.clip_to_window_via_transform(&b).truncate();
        let c_ndc = self.clip_to_window_via_transform(&c).truncate();
        let to_ndc = Vector3::new(1.0 / self.width, 1.0 / self.height, 1.0);
        let plane: Plane<f64> = Plane::from_points(
            Point3::from_vec(a_ndc.mul_element_wise(to_ndc)),
            Point3::from_vec(b_ndc.mul_element_wise(to_ndc)),
            Point3::from_vec(c_ndc.mul_element_wise(to_ndc)),
        )
        .unwrap();
        println!("{:?}", &plane);

        let points = plane.intersection_points_aabb3(&Aabb3::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 1.0),
        ));

        let from_ndc = Vector3::new(self.width, self.height, 1.0);
        let vec = points
            .iter()
            .map(|point| self.window_to_world(&point.mul_element_wise(from_ndc), &view_proj))
            .collect::<Vec<_>>();

        let min_x = vec
            .iter()
            .map(|point| point.x)
            .min_by(|a, b| a.partial_cmp(b).unwrap())?;
        let min_y = vec
            .iter()
            .map(|point| point.y)
            .min_by(|a, b| a.partial_cmp(b).unwrap())?;
        let max_x = vec
            .iter()
            .map(|point| point.x)
            .max_by(|a, b| a.partial_cmp(b).unwrap())?;
        let max_y = vec
            .iter()
            .map(|point| point.y)
            .max_by(|a, b| a.partial_cmp(b).unwrap())?;
        Some(Aabb2::new(
            Point2::new(min_x, min_y),
            Point2::new(max_x, max_y),
        ))
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
    
    use cgmath::{AbsDiffEq, Matrix4, SquareMatrix, Vector2, Vector3, Vector4};

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
        let window = camera.clip_to_window_via_transform(&clip);
        println!("window (matrix): {:?}", window);

        // --------- nalgebra

        println!(
            "r world (nalgebra): {:?}",
            Camera::window_to_world_nalgebra(&window.truncate(), &view_proj, width, height)
        );

        // -------- far vs. near plane trick

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
        assert!(Vector3::new(world_pos.x, world_pos.y, world_pos.z).abs_diff_eq(&unprojected, 0.05));

        // ---- test for unproject

        let window = Vector2::new(960.0, 631.0); // 0, 4096: passt nicht
                                                 //let window = Vector2::new(962.0, 1.0); // 0, 300: passt nicht
                                                 //let window = Vector2::new(960.0, 540.0); // 0, 0 passt
        let near_world = camera.window_to_world(&Vector3::new(window.x, window.y, 0.0), &view_proj);

        let far_world = camera.window_to_world(&Vector3::new(window.x, window.y, 1.0), &view_proj);

        // for z = 0 in world coordinates
        let u = -near_world.z / (far_world.z - near_world.z);
        println!("u: {:?}", u);
        let unprojected = near_world + u * (far_world - near_world);
        println!("unprojected: {:?}", unprojected);
        // ----

        //assert!(reverse_world.abs_diff_eq(&world_pos, 0.05))
    }
}
