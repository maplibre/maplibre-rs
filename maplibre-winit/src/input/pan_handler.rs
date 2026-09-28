use std::time::Duration;

use cgmath::{EuclideanSpace, Point2, Vector2, Zero};
use instant::Instant;
use maplibre::context::MapContext;
use winit::event::{ElementState, MouseButton};

use super::{
    inertia::PanInertia,
    projection::{center_pixel, pan_globe_by_pixels, pan_plane_by_pixels},
    UpdateState,
};

#[derive(Default)]
pub struct PanHandler {
    window_position: Option<Vector2<f64>>,
    last_window_position: Option<Vector2<f64>>,
    start_window_position: Option<Vector2<f64>>,
    start_camera_position: Option<Vector2<f64>>,
    is_panning: bool,
    inertia: PanInertia,
}

impl UpdateState for PanHandler {
    fn update_state(
        &mut self,
        MapContext {
            style, view_state, ..
        }: &mut MapContext,
        _dt: Duration,
    ) {
        let now = Instant::now();
        if !self.is_panning {
            if let Some(delta) = self.inertia.step(now) {
                let center = center_pixel(view_state);
                if !pan_globe_by_pixels(style, view_state, center, delta) {
                    pan_plane_by_pixels(view_state, center, delta);
                }
            }
            return;
        }

        let (Some(window_position), Some(start_window_position)) =
            (self.window_position, self.start_window_position)
        else {
            return;
        };
        let delta = window_position - self.last_window_position.unwrap_or(window_position);
        self.last_window_position = Some(window_position);
        self.inertia.record(now, delta);

        if pan_globe_by_pixels(style, view_state, window_position, delta) {
            return;
        }

        let view_proj = view_state.view_projection();
        let inverted_view_proj = view_proj.invert();

        let delta = if let (Some(start), Some(current)) = (
            view_state.window_to_world_at_ground(
                &start_window_position,
                &inverted_view_proj,
                false,
            ),
            view_state.window_to_world_at_ground(&window_position, &inverted_view_proj, false),
        ) {
            start - current
        } else {
            Vector2::zero()
        };

        if self.start_camera_position.is_none() {
            self.start_camera_position = Some(view_state.camera().position().to_vec());
        }

        if let Some(start_camera_position) = self.start_camera_position {
            view_state.camera_mut().move_to(Point2::from_vec(
                start_camera_position + Vector2::new(delta.x, delta.y),
            ));
        }
    }
}

impl PanHandler {
    pub fn process_touch_start(&mut self, window_position: &Vector2<f64>) -> bool {
        self.begin(Some(*window_position));
        true
    }

    pub fn process_touch_end(&mut self) -> bool {
        self.end();
        true
    }

    pub fn process_window_position(&mut self, window_position: &Vector2<f64>, touch: bool) -> bool {
        if !self.is_panning && !touch {
            self.start_window_position = Some(*window_position);
            self.last_window_position = Some(*window_position);
            self.window_position = Some(*window_position);
        } else {
            self.window_position = Some(*window_position);
        }

        true
    }

    pub fn process_mouse_key_press(&mut self, key: &MouseButton, state: &ElementState) -> bool {
        if *key != MouseButton::Left {
            return false;
        }

        if *state == ElementState::Pressed {
            self.begin(None);
        } else {
            self.end();
        }
        true
    }

    fn begin(&mut self, window_position: Option<Vector2<f64>>) {
        self.inertia.cancel();
        self.is_panning = true;
        if let Some(window_position) = window_position {
            self.start_window_position = Some(window_position);
            self.last_window_position = Some(window_position);
            self.window_position = Some(window_position);
        }
    }

    fn end(&mut self) {
        self.inertia.release(Instant::now());
        self.start_camera_position = None;
        self.start_window_position = None;
        self.last_window_position = None;
        self.window_position = None;
        self.is_panning = false;
    }
}
