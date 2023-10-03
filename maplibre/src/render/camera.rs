//! Main camera

use std::convert::Into;

use cgmath::{num_traits::clamp, prelude::*, *};

use crate::util::SignificantlyDifferent;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f64> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[rustfmt::skip]
pub const FLIP_Y: Matrix4<f64> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0, 
    0.0, -1.0, 0.0, 0.0, 
    0.0, 0.0, 1.0, 0.0, 
    0.0, 0.0, 0.0, 1.0,
);

#[derive(Debug, Clone, Copy)]
pub struct ViewProjection(pub Matrix4<f64>);

impl ViewProjection {
    #[tracing::instrument(skip_all)]
    pub fn invert(&self) -> InvertedViewProjection {
        InvertedViewProjection(self.0.invert().expect("Unable to invert view projection"))
    }

    pub fn project(&self, vector: Vector4<f64>) -> Vector4<f64> {
        self.0 * vector
    }

    #[tracing::instrument(skip_all)]
    pub fn to_model_view_projection(&self, projection: Matrix4<f64>) -> ModelViewProjection {
        ModelViewProjection(self.0 * projection)
    }

    pub fn downcast(&self) -> Matrix4<f32> {
        self.0
            .cast::<f32>()
            .expect("Unable to cast view projection to f32")
    }
}

pub struct InvertedViewProjection(Matrix4<f64>);

impl InvertedViewProjection {
    pub fn project(&self, vector: Vector4<f64>) -> Vector4<f64> {
        self.0 * vector
    }
}

pub struct ModelViewProjection(Matrix4<f64>);

impl ModelViewProjection {
    pub fn downcast(&self) -> Matrix4<f32> {
        self.0
            .cast::<f32>()
            .expect("Unable to cast view projection to f32")
    }
}

const MIN_PITCH: Deg<f64> = Deg(-30.0);
const MAX_PITCH: Deg<f64> = Deg(30.0);

const MIN_YAW: Deg<f64> = Deg(-30.0);
const MAX_YAW: Deg<f64> = Deg(30.0);

#[derive(Debug, Clone)]
pub struct Camera {
    position: Point2<f64>,
    yaw: Rad<f64>,
    pitch: Rad<f64>,
    roll: Rad<f64>,
}

impl SignificantlyDifferent for Camera {
    type Epsilon = f64;

    fn ne(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        self.position.abs_diff_ne(&other.position, epsilon)
            || self.yaw.abs_diff_ne(&other.yaw, epsilon)
            || self.pitch.abs_diff_ne(&other.pitch, epsilon)
            || self.roll.abs_diff_ne(&other.roll, epsilon)
    }
}

impl Camera {
    pub fn new<V: Into<Point2<f64>>, Y: Into<Rad<f64>>, P: Into<Rad<f64>>>(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
            roll: Rad::zero(), // TODO: initialize
        }
    }

    pub fn calc_matrix(&self, camera_height: f64) -> Matrix4<f64> {
        Matrix4::from_translation(Vector3::new(0.0, 0.0, -camera_height))
            * Matrix4::from_angle_x(self.pitch)
            * Matrix4::from_angle_y(self.yaw)
            * Matrix4::from_angle_z(self.roll)
            * Matrix4::from_translation(Vector3::new(-self.position.x, -self.position.y, 0.0))
    }

    pub fn position(&self) -> Point2<f64> {
        self.position
    }

    pub fn get_yaw(&self) -> Rad<f64> {
        self.yaw
    }

    pub fn yaw<P: Into<Rad<f64>>>(&mut self, delta: P) {
        let new_yaw = self.yaw + delta.into();

        if new_yaw <= MAX_YAW.into() && new_yaw >= MIN_YAW.into() {
            self.yaw = new_yaw;
        }
    }

    pub fn get_roll(&self) -> Rad<f64> {
        self.roll
    }

    pub fn roll<P: Into<Rad<f64>>>(&mut self, delta: P) {
        self.roll += delta.into();
    }

    pub fn get_pitch(&self) -> Rad<f64> {
        self.pitch
    }

    pub fn pitch<P: Into<Rad<f64>>>(&mut self, delta: P) {
        let new_pitch = self.pitch + delta.into();

        if new_pitch <= MAX_PITCH.into() && new_pitch >= MIN_PITCH.into() {
            self.pitch = new_pitch;
        }
    }

    pub fn move_relative(&mut self, delta: Vector2<f64>) {
        self.position += delta;
    }

    pub fn move_to(&mut self, new_position: Point2<f64>) {
        self.position = new_position;
    }

    pub fn position_vector(&self) -> Vector2<f64> {
        self.position.to_vec()
    }

    pub fn to_3d(&self, camera_height: f64) -> Point3<f64> {
        Point3::new(self.position.x, self.position.y, camera_height)
    }
    pub fn set_yaw<P: Into<Rad<f64>>>(&mut self, yaw: P) {
        let new_yaw = yaw.into();
        let max: Rad<_> = MAX_YAW.into();
        let min: Rad<_> = MIN_YAW.into();
        self.yaw = Rad(new_yaw.0.min(max.0).max(min.0))
    }
    pub fn set_pitch<P: Into<Rad<f64>>>(&mut self, pitch: P) {
        let new_pitch = pitch.into();
        let max: Rad<_> = MAX_PITCH.into();
        let min: Rad<_> = MIN_PITCH.into();
        self.pitch = Rad(new_pitch.0.min(max.0).max(min.0))
    }
    pub fn set_roll<P: Into<Rad<f64>>>(&mut self, roll: P) {
        self.roll = roll.into();
    }
}

#[derive(PartialEq, Copy, Clone, Default)]
pub struct EdgeInsets {
    pub top: f64,
    pub bottom: f64,
    pub left: f64,
    pub right: f64,
}

impl EdgeInsets {
    /**
     * Utility method that computes the new apprent center or vanishing point after applying insets.
     * This is in pixels and with the top left being (0.0) and +y being downwards.
     *
     * @param {number} width the width
     * @param {number} height the height
     * @returns {Point} the point
     * @memberof EdgeInsets
     */
    pub fn center(&self, width: f64, height: f64) -> Point2<f64> {
        // Clamp insets so they never overflow width/height and always calculate a valid center
        let x = clamp((self.left + width - self.right) / 2.0, 0.0, width);
        let y = clamp((self.top + height - self.bottom) / 2.0, 0.0, height);

        return Point2::new(x, y);
    }
}

pub struct Perspective {
    fovy: Rad<f64>,
}

impl Perspective {
    pub fn new<F: Into<Rad<f64>>>(fovy: F) -> Self {
        let rad = fovy.into();
        Self { fovy: rad }
    }

    pub fn fovy(&self) -> Rad<f64> {
        self.fovy
    }

    pub fn calc_matrix(&self, aspect: f64, near_z: f64, far_z: f64) -> Matrix4<f64> {
        perspective(self.fovy, aspect, near_z, far_z)
    }

    pub fn calc_matrix_with_center(
        &self,
        width: f64,
        height: f64,
        near_z: f64,
        far_z: f64,
        center_offset: Point2<f64>,
    ) -> Matrix4<f64> {
        let aspect = width / height;

        // from projection.rs
        let half_fovy = self.fovy / 2.0;
        let ymax = near_z * half_fovy.tan();

        //let xmax = ymax * aspect;
        let half_fovx = Rad(2.0 * (half_fovy.tan() * aspect).atan()) / 2.0;
        let xmax = near_z * half_fovx.tan();

        let offset_x = center_offset.x * 2.0 / width; // TODO - or + does not matter
        let offset_y = center_offset.y * 2.0 / height;
        frustum(
            // https://webglfundamentals.org/webgl/lessons/webgl-qna-how-can-i-move-the-perspective-vanishing-point-from-the-center-of-the-canvas-.html
            xmax * (-1.0 + offset_x), /* = -xmax + (center_offset.x * screen_to_near_factor_x)
                                                 where:
                                                  screen_to_near_factor_x = near_width / width
                                                  where:
                                                    near_width = xmax * 2.0
                                      */
            xmax * (1.0 + offset_x),
            ymax * (-1.0 + offset_y),
            ymax * (1.0 + offset_y),
            near_z,
            far_z,
        )
    }
}

#[cfg(test)]
mod tests {
    /*
    use cgmath::{AbsDiffEq, Vector2, Vector3, Vector4};

    use super::{Camera, Perspective};
    use crate::render::camera::{InvertedViewProjection, ViewProjection};

    #[test]
    fn test() {
        let width = 1920.0;
        let height = 1080.0;
        let camera = Camera::new((0.0, 5.0, 5000.0), cgmath::Deg(-90.0), cgmath::Deg(45.0));
        // 4732.561319582916
        let perspective = Perspective::new(cgmath::Deg(45.0));
        let view_proj: ViewProjection = camera.calc_view_proj(&perspective);
        let inverted_view_proj: InvertedViewProjection = view_proj.invert();

        let world_pos: Vector4<f64> = Vector4::new(0.0, 0.0, 0.0, 1.0);
        let clip = view_proj.project(world_pos);

        let origin_clip_space = view_proj.project(Vector4::new(0.0, 0.0, 0.0, 1.0));
        println!("origin w in clip space: {:?}", origin_clip_space.w);

        println!("world_pos: {world_pos:?}");
        println!("clip: {clip:?}");
        println!("world_pos: {:?}", view_proj.invert().project(clip));

        println!("window: {:?}", camera.clip_to_window_vulkan(&clip));
        let window = camera.clip_to_window(&clip);
        println!("window (matrix): {window:?}");

        // --------- nalgebra

        println!(
            "r world (nalgebra): {:?}",
            Camera::window_to_world_nalgebra(
                &window.truncate(),
                &inverted_view_proj,
                width,
                height
            )
        );

        // -------- far vs. near plane trick

        let near_world = Camera::window_to_world_nalgebra(
            &Vector3::new(window.x, window.y, 0.0),
            &inverted_view_proj,
            width,
            height,
        );

        let far_world = Camera::window_to_world_nalgebra(
            &Vector3::new(window.x, window.y, 1.0),
            &inverted_view_proj,
            width,
            height,
        );

        // for z = 0 in world coordinates
        let u = -near_world.z / (far_world.z - near_world.z);
        println!("u: {u:?}");
        let unprojected = near_world + u * (far_world - near_world);
        println!("unprojected: {unprojected:?}");
        assert!(Vector3::new(world_pos.x, world_pos.y, world_pos.z).abs_diff_eq(&unprojected, 0.05));

        // ---- test for unproject

        let window = Vector2::new(960.0, 631.0); // 0, 4096: passt nicht
                                                 //let window = Vector2::new(962.0, 1.0); // 0, 300: passt nicht
                                                 //let window = Vector2::new(960.0, 540.0); // 0, 0 passt
        let near_world =
            camera.window_to_world(&Vector3::new(window.x, window.y, 0.0), &inverted_view_proj);

        let far_world =
            camera.window_to_world(&Vector3::new(window.x, window.y, 1.0), &inverted_view_proj);

        // for z = 0 in world coordinates
        let u = -near_world.z / (far_world.z - near_world.z);
        println!("u: {u:?}");
        let unprojected = near_world + u * (far_world - near_world);
        println!("unprojected: {unprojected:?}");
        // ----

        //assert!(reverse_world.abs_diff_eq(&world_pos, 0.05))
    }*/
}
