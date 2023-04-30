use cgmath::num_traits::clamp;
use cgmath::prelude::*;
use cgmath::*;
use std::f64::consts::{FRAC_PI_2, PI};
use std::ops::{Deref, DerefMut};

use crate::render::camera::{EdgeInsets, InvertedViewProjection, FLIP_Y, OPENGL_TO_WGPU_MATRIX};
use crate::util::math::{bounds_from_points, Aabb2, Aabb3, Plane};
use crate::{
    coords::{ViewRegion, WorldCoords, Zoom, ZoomLevel},
    render::camera::{Camera, Perspective, ViewProjection},
    util::ChangeObserver,
    window::WindowSize,
};

const VIEW_REGION_PADDING: i32 = 1;
const MAX_N_TILES: usize = 128;

pub struct ViewState {
    zoom: ChangeObserver<Zoom>,
    camera: ChangeObserver<Camera>,
    perspective: Perspective,

    width: f64,
    height: f64,
    edge_insets: EdgeInsets,
}

impl ViewState {
    pub fn new<F: Into<Rad<f64>>, P: Into<Deg<f64>>>(
        window_size: WindowSize,
        position: WorldCoords,
        zoom: Zoom,
        pitch: P,
        fovy: F,
    ) -> Self {
        let camera = Camera::new((position.x, position.y), Deg(-90.0), pitch.into());

        let perspective = Perspective::new(fovy);

        Self {
            zoom: ChangeObserver::new(zoom),
            camera: ChangeObserver::new(camera),
            perspective,
            width: window_size.width() as f64,
            height: window_size.height() as f64,
            edge_insets: EdgeInsets {
                top: 0.0,
                bottom: 0.0,
                left: 0.0,
                right: 0.0,
            },
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width as f64;
        self.height = height as f64;
    }

    pub fn create_view_region(&self, visible_level: ZoomLevel) -> Option<ViewRegion> {
        self.view_region_bounding_box(&self.view_projection().invert())
            .map(|bounding_box| {
                ViewRegion::new(
                    bounding_box,
                    VIEW_REGION_PADDING,
                    MAX_N_TILES,
                    *self.zoom,
                    visible_level,
                )
            })
    }

    /// This function matches how maplibre-gl-js implements perspective and cameras.
    /// Therefore it has the same bugs as maplibre-gl-js (2023-04): https://github.com/maplibre/maplibre-gl-js/issues/1655
    /// It implements also already some elevation based clipping. This is not used here.
    ///
    /// Last verified to match maplibre-gl-js on 2023-04-15.
    #[tracing::instrument(skip_all)]
    pub fn view_projection(&self) -> ViewProjection {
        let width = self.width;
        let height = self.height;
        let aspect = width / height;
        let pitch = self.camera.pitch();

        let center = self.edge_insets.center(width, height);
        let offset = center - Vector2::new(width, height) / 2.0;

        let fovy = self.perspective.fovy();
        let half_fov = fovy / 2.0;
        let camera_to_center_distance = 0.5 / half_fov.tan() * height;

        // TODO (only relevant for 3D terrain) let center: LatLon = 0.0;
        // TODO (only relevant for 3D terrain) let pixel_per_meter = center.mercatorZfromAltitude(1) * this.worldSize;

        // TODO: labelPlaneMatrix https://github.com/maplibre/maplibre-gl-js/blob/80e232a64716779bfff841dbc18fddc1f51535ad/src/geo/transform.ts#L833-L836
        // TODO: glCoordMatrix https://github.com/maplibre/maplibre-gl-js/blob/80e232a64716779bfff841dbc18fddc1f51535ad/src/geo/transform.ts#L838-L842

        // TODO: cameraToSeaLevelDistance https://github.com/maplibre/maplibre-gl-js/blob/80e232a64716779bfff841dbc18fddc1f51535ad/src/geo/transform.ts#L844-L847
        let lowest_plane = camera_to_center_distance; // TODO const lowestPlane = this._elevation < 0 ? this.cameraToCenterDistance : this.cameraToSeaLevelDistance;

        // Find the distance from the center point [width/2 + offset.x, height/2 + offset.y] to the
        // center top point [width/2 + offset.x, 0] in Z units, using the law of sines.
        // 1 Z unit is equivalent to 1 horizontal px at the center of the map
        // (the distance between[width/2, height/2] and [width/2 + 1, height/2])
        let ground_angle = Rad(FRAC_PI_2) + pitch;
        let fov_above_center = fovy * (0.5 + offset.y / height);
        let top_half_surface_distance = fov_above_center.sin() * camera_to_center_distance
            / clamp(
                Rad(PI) - ground_angle - fov_above_center,
                Rad(0.01),
                Rad(PI - 0.01),
            )
            .sin();

        let horizon = (Rad(FRAC_PI_2) - pitch).tan() * camera_to_center_distance * 0.85;
        let horizon_angle = Rad((horizon / camera_to_center_distance).atan());
        let fov_center_to_horizon = horizon_angle * 2.0 * (0.5 + offset.y / (horizon * 2.0));
        let top_half_surface_distance_horizon = (fov_center_to_horizon).sin() * lowest_plane
            / (clamp(
                Rad(PI) - ground_angle - fov_center_to_horizon,
                Rad(0.01),
                Rad(PI - 0.01),
            ))
            .sin();

        // Calculate z distance of the farthest fragment that should be rendered.
        // Add a bit extra to avoid precision problems when a fragment's distance is exactly `furthest_distance`
        let top_half_min_distance =
            top_half_surface_distance.min(top_half_surface_distance_horizon);
        let far_z = ((Rad(FRAC_PI_2) - pitch).cos() * top_half_min_distance + lowest_plane) * 1.01;

        // The larger the value of near_z is
        // - the more depth precision is available for features (good)
        // - clipping starts appearing sooner when the camera is close to 3d features (bad)
        //
        // Smaller values worked well for mapbox-gl-js but deckgl was encountering precision issues
        // when rendering it's layers using custom layers. This value was experimentally chosen and
        // seems to solve z-fighting issues in deckgl while not clipping buildings too close to the camera.
        //
        // In tile.vertex.wgsl we are setting each layer's final `z` in ndc space to `z_index`.
        // This means that regardless of the `znear` value all layers will be rendered as part
        // of the near plane.
        // These values have been selected experimentally:
        // https://www.sjbaker.org/steve/omniv/love_your_z_buffer.html
        let near_z = height / 50.0;

        let mut perspective = self.perspective.calc_matrix(aspect, near_z, far_z);

        // Apply center of perspective offset, in order to move the vanishing point
        perspective.z[0] = -offset.x * 2.0 / width;
        perspective.z[1] = offset.y * 2.0 / height;

        let view_projection = perspective
            // Apply camera and move camera away from ground
            * self.camera.calc_matrix(camera_to_center_distance);

        // TODO for the below TODOs, check GitHub blame to get an idea of what these matrices are used for!

        // TODO mercatorMatrix https://github.com/maplibre/maplibre-gl-js/blob/80e232a64716779bfff841dbc18fddc1f51535ad/src/geo/transform.ts#L891-L893

        // TODO scale vertically to meters per pixel (inverse of ground resolution):
        // TODO (only relevant for 3D terrain) mat4.scale(view_projection, view_projection, [1, 1, this._pixelPerMeter]);

        // TODO pixelMatrix https://github.com/maplibre/maplibre-gl-js/blob/80e232a64716779bfff841dbc18fddc1f51535ad/src/geo/transform.ts#L898-L899
        // TODO invProjMatrix, projMatrix https://github.com/maplibre/maplibre-gl-js/blob/80e232a64716779bfff841dbc18fddc1f51535ad/src/geo/transform.ts#L901-L904
        // TODO (only relevant for 3D terrain) pixelMatrix3D https://github.com/maplibre/maplibre-gl-js/blob/80e232a64716779bfff841dbc18fddc1f51535ad/src/geo/transform.ts#L906-L907
        // TODO alignedProjMatrix https://github.com/maplibre/maplibre-gl-js/blob/80e232a64716779bfff841dbc18fddc1f51535ad/src/geo/transform.ts#L909-L921
        // TODO pixelMatrixInverse https://github.com/maplibre/maplibre-gl-js/blob/80e232a64716779bfff841dbc18fddc1f51535ad/src/geo/transform.ts#L923-L926

        ViewProjection(FLIP_Y * OPENGL_TO_WGPU_MATRIX * view_projection)
    }

    pub fn zoom(&self) -> Zoom {
        *self.zoom
    }

    pub fn did_zoom_change(&self) -> bool {
        self.zoom.did_change(0.05)
    }

    pub fn update_zoom(&mut self, new_zoom: Zoom) {
        *self.zoom = new_zoom;
        log::info!("zoom: {}", new_zoom);
    }

    pub fn camera(&self) -> &Camera {
        self.camera.deref()
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        self.camera.deref_mut()
    }

    pub fn did_camera_change(&self) -> bool {
        self.camera.did_change(0.05)
    }

    pub fn update_references(&mut self) {
        self.camera.update_reference();
        self.zoom.update_reference();
    }

    /// A transform which can be used to transform between clip and window space.
    /// Adopted from [here](https://docs.microsoft.com/en-us/windows/win32/direct3d9/viewports-and-clipping#viewport-rectangle) (Direct3D).
    fn clip_to_window_transform(&self) -> Matrix4<f64> {
        let min_depth = 0.0;
        let max_depth = 1.0;
        let x = 0.0;
        let y = 0.0;
        let ox = x + self.width / 2.0;
        let oy = y + self.height / 2.0;
        let oz = min_depth;
        let pz = max_depth - min_depth;
        Matrix4::from_cols(
            Vector4::new(self.width / 2.0, 0.0, 0.0, 0.0),
            Vector4::new(0.0, -self.height / 2.0, 0.0, 0.0),
            Vector4::new(0.0, 0.0, pz, 0.0),
            Vector4::new(ox, oy, oz, 1.0),
        )
    }

    /// Transforms coordinates in clip space to window coordinates.
    ///
    /// Adopted from [here](https://docs.microsoft.com/en-us/windows/win32/dxtecharts/the-direct3d-transformation-pipeline) (Direct3D).
    fn clip_to_window(&self, clip: &Vector4<f64>) -> Vector4<f64> {
        #[rustfmt::skip]
            let ndc = Vector4::new(
            clip.x / clip.w,
            clip.y / clip.w,
            clip.z / clip.w,
            1.0
        );

        self.clip_to_window_transform() * ndc
    }
    /// Alternative implementation to `clip_to_window`. Transforms coordinates in clip space to
    /// window coordinates.
    ///
    /// Adopted from [here](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkViewport.html)
    /// and [here](https://matthewwellings.com/blog/the-new-vulkan-coordinate-system/) (Vulkan).
    fn clip_to_window_vulkan(&self, clip: &Vector4<f64>) -> Vector3<f64> {
        #[rustfmt::skip]
            let ndc = Vector4::new(
            clip.x / clip.w,
            clip.y / clip.w,
            clip.z / clip.w,
            1.0
        );

        let min_depth = 0.0;
        let max_depth = 1.0;

        let x = 0.0;
        let y = 0.0;
        let ox = x + self.width / 2.0;
        let oy = y + self.height / 2.0;
        let oz = min_depth;
        let px = self.width;
        let py = self.height;
        let pz = max_depth - min_depth;
        let xd = ndc.x;
        let yd = ndc.y;
        let zd = ndc.z;
        Vector3::new(px / 2.0 * xd + ox, py / 2.0 * yd + oy, pz * zd + oz)
    }

    /// Order of transformations reversed: https://computergraphics.stackexchange.com/questions/6087/screen-space-coordinates-to-eye-space-conversion/6093
    /// `w` is lost.
    ///
    /// OpenGL explanation: https://www.khronos.org/opengl/wiki/Compute_eye_space_from_window_space#From_window_to_ndc
    fn window_to_world(
        &self,
        window: &Vector3<f64>,
        inverted_view_proj: &InvertedViewProjection,
    ) -> Vector3<f64> {
        #[rustfmt::skip]
            let fixed_window = Vector4::new(
            window.x,
            window.y,
            window.z,
            1.0
        );

        let ndc = self.clip_to_window_transform().invert().unwrap() * fixed_window;
        let unprojected = inverted_view_proj.project(ndc);

        Vector3::new(
            unprojected.x / unprojected.w,
            unprojected.y / unprojected.w,
            unprojected.z / unprojected.w,
        )
    }

    /// Alternative implementation to `window_to_world`
    ///
    /// Adopted from [here](https://docs.rs/nalgebra-glm/latest/src/nalgebra_glm/ext/matrix_projection.rs.html#164-181).
    fn window_to_world_nalgebra(
        window: &Vector3<f64>,
        inverted_view_proj: &InvertedViewProjection,
        width: f64,
        height: f64,
    ) -> Vector3<f64> {
        let pt = Vector4::new(
            2.0 * (window.x - 0.0) / width - 1.0,
            2.0 * (height - window.y - 0.0) / height - 1.0,
            window.z,
            1.0,
        );
        let unprojected = inverted_view_proj.project(pt);

        Vector3::new(
            unprojected.x / unprojected.w,
            unprojected.y / unprojected.w,
            unprojected.z / unprojected.w,
        )
    }

    /// Gets the world coordinates for the specified `window` coordinates on the `z=0` plane.
    pub fn window_to_world_at_ground(
        &self,
        window: &Vector2<f64>,
        inverted_view_proj: &InvertedViewProjection,
        bound: bool,
    ) -> Option<Vector2<f64>> {
        let near_world =
            self.window_to_world(&Vector3::new(window.x, window.y, 0.0), inverted_view_proj);

        let far_world =
            self.window_to_world(&Vector3::new(window.x, window.y, 1.0), inverted_view_proj);

        // for z = 0 in world coordinates
        // Idea comes from: https://dondi.lmu.build/share/cg/unproject-explained.pdf
        let u = -near_world.z / (far_world.z - near_world.z);
        if !bound || (0.0..=1.0).contains(&u) {
            let result = near_world + u * (far_world - near_world);
            Some(Vector2::new(result.x, result.y))
        } else {
            None
        }
    }

    /// Calculates an [`Aabb2`] bounding box which contains at least the visible area on the `z=0`
    /// plane. One can think of it as being the bounding box of the geometry which forms the
    /// intersection between the viewing frustum and the `z=0` plane.
    ///
    /// This implementation works in the world 3D space. It casts rays from the corners of the
    /// window to calculate intersections points with the `z=0` plane. Then a bounding box is
    /// calculated.
    ///
    /// *Note:* It is possible that no such bounding box exists. This is the case if the `z=0` plane
    /// is not in view.
    pub fn view_region_bounding_box(
        &self,
        inverted_view_proj: &InvertedViewProjection,
    ) -> Option<Aabb2<f64>> {
        let screen_bounding_box = [
            Vector2::new(0.0, 0.0),
            Vector2::new(self.width, 0.0),
            Vector2::new(self.width, self.height),
            Vector2::new(0.0, self.height),
        ]
        .map(|point| self.window_to_world_at_ground(&point, inverted_view_proj, false));

        let (min, max) = bounds_from_points(
            screen_bounding_box
                .into_iter()
                .flatten()
                .map(|point| [point.x, point.y]),
        )?;

        Some(Aabb2::new(Point2::from(min), Point2::from(max)))
    }
    /// An alternative implementation for `view_bounding_box`.
    ///
    /// This implementation works in the NDC space. We are creating a plane in the world 3D space.
    /// Then we are transforming it to the NDC space. In NDC space it is easy to calculate
    /// the intersection points between an Aabb3 and a plane. The resulting Aabb2 is returned.
    pub fn view_region_bounding_box_ndc(&self) -> Option<Aabb2<f64>> {
        let view_proj = self.view_projection();
        let a = view_proj.project(Vector4::new(0.0, 0.0, 0.0, 1.0));
        let b = view_proj.project(Vector4::new(1.0, 0.0, 0.0, 1.0));
        let c = view_proj.project(Vector4::new(1.0, 1.0, 0.0, 1.0));

        let a_ndc = self.clip_to_window(&a).truncate();
        let b_ndc = self.clip_to_window(&b).truncate();
        let c_ndc = self.clip_to_window(&c).truncate();
        let to_ndc = Vector3::new(1.0 / self.width, 1.0 / self.height, 1.0);
        let plane: Plane<f64> = Plane::from_points(
            Point3::from_vec(a_ndc.mul_element_wise(to_ndc)),
            Point3::from_vec(b_ndc.mul_element_wise(to_ndc)),
            Point3::from_vec(c_ndc.mul_element_wise(to_ndc)),
        )?;

        let points = plane.intersection_points_aabb3(&Aabb3::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 1.0),
        ));

        let inverted_view_proj = view_proj.invert();

        let from_ndc = Vector3::new(self.width, self.height, 1.0);
        let vec = points
            .iter()
            .map(|point| {
                self.window_to_world(&point.mul_element_wise(from_ndc), &inverted_view_proj)
            })
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
