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
    pub fn fovx(&self, width: f64, height: f64) -> Rad<f64> {
        let aspect = width / height;
        Rad(2.0 * ((self.fovy / 2.0).tan() * aspect).atan())
    }

    pub fn y_tan(&self) -> f64 {
        let half_fovy = self.fovy / 2.0;
        half_fovy.tan()
    }
    pub fn x_tan(&self, width: f64, height: f64) -> f64 {
        let half_fovx = self.fovx(width, height) / 2.0;
        half_fovx.tan()
    }

    pub fn offset_x(&self, center_offset: Point2<f64>, width: f64) -> f64 {
        center_offset.x * 2.0 / width
    }

    pub fn offset_y(&self, center_offset: Point2<f64>, height: f64) -> f64 {
        center_offset.y * 2.0 / height
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
        let ymax = near_z * self.y_tan();

        //TODO maybe just: let xmax = ymax * aspect;
        let xmax = near_z * self.x_tan(width, height);

        let offset_x = self.offset_x(center_offset, width);
        let offset_y = self.offset_y(center_offset, height);
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
