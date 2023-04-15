//! Main camera

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

#[derive(Debug)]
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

const MIN_PITCH: Rad<f64> = Rad(-0.5);
const MAX_PITCH: Rad<f64> = Rad(0.5);

#[derive(Debug, Clone)]
pub struct Camera {
    position: Point3<f64>, // The z axis never changes, the zoom is used instead
    yaw: Rad<f64>,
    pitch: Rad<f64>,
}

impl SignificantlyDifferent for Camera {
    type Epsilon = f64;

    fn ne(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        self.position.abs_diff_ne(&other.position, epsilon)
            || self.yaw.abs_diff_ne(&other.yaw, epsilon)
            || self.pitch.abs_diff_ne(&other.pitch, epsilon)
    }
}

impl Camera {
    pub fn new<V: Into<Point3<f64>>, Y: Into<Rad<f64>>, P: Into<Rad<f64>>>(
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

    pub fn calc_matrix(&self) -> Matrix4<f64> {
        Matrix4::look_to_rh(
            self.position,
            Vector3::new(self.yaw.cos(), self.pitch.sin(), self.yaw.sin()).normalize(),
            Vector3::unit_y(),
        )
    }

    pub fn position(&self) -> Point3<f64> {
        self.position
    }

    pub fn yaw(&self) -> Rad<f64> {
        self.yaw
    }

    pub fn rotate<P: Into<Rad<f64>>>(&mut self, delta: P) {
        self.yaw += delta.into();
    }

    pub fn pitch(&self) -> Rad<f64> {
        self.pitch
    }

    pub fn tilt<P: Into<Rad<f64>>>(&mut self, delta: P) {
        let new_pitch = self.pitch + delta.into();

        if new_pitch <= MAX_PITCH && new_pitch >= MIN_PITCH {
            self.pitch = new_pitch;
        }
    }

    pub fn move_relative(&mut self, delta: Vector3<f64>) {
        self.position += delta;
    }

    pub fn move_to(&mut self, new_position: Point3<f64>) {
        self.position = new_position;
    }

    pub fn position_vector(&self) -> Vector3<f64> {
        self.position.to_vec()
    }

    pub fn homogenous_position(&self) -> Vector4<f64> {
        self.position.to_homogeneous()
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

    // Adopted from https://github.com/maplibre/maplibre-gl-js/blob/80e232a64716779bfff841dbc18fddc1f51535ad/src/geo/transform.ts#L827-L879
    pub fn calc_matrix(&self, aspect: f64, near_z: f64, far_z: f64) -> Matrix4<f64> {
        perspective(self.fovy, aspect, near_z, far_z)
    }
}

#[cfg(test)]
mod tests {
    use cgmath::{AbsDiffEq, Vector2, Vector3, Vector4};

    use super::{Camera, Perspective};
    use crate::render::camera::{EdgeInsets, InvertedViewProjection, ViewProjection};

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
        let perspective = Perspective::new(width as u32, height as u32, cgmath::Deg(45.0), &camera);
        let view_proj: ViewProjection = camera.calc_view_proj(&perspective);
        let inverted_view_proj: InvertedViewProjection = view_proj.invert();

        let world_pos: Vector4<f64> = Vector4::new(0.0, 0.0, 0.0, 1.0);
        let clip = view_proj.project(world_pos);

        let origin_clip_space = view_proj.project(Vector4::new(0.0, 0.0, 0.0, 1.0));
        println!("origin w in clip space: {:?}", origin_clip_space.w);

        println!("world_pos: {:?}", world_pos);
        println!("clip: {:?}", clip);
        println!("world_pos: {:?}", view_proj.invert().project(clip));

        println!("window: {:?}", camera.clip_to_window_vulkan(&clip));
        let window = camera.clip_to_window(&clip);
        println!("window (matrix): {:?}", window);

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
        println!("u: {:?}", u);
        let unprojected = near_world + u * (far_world - near_world);
        println!("unprojected: {:?}", unprojected);
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
        println!("u: {:?}", u);
        let unprojected = near_world + u * (far_world - near_world);
        println!("unprojected: {:?}", unprojected);
        // ----

        //assert!(reverse_world.abs_diff_eq(&world_pos, 0.05))
    }
}
