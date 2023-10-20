use std::time::Duration;

use cgmath::Vector2;
use maplibre::{
    context::MapContext, coords::WorldCoords, io::geometry_index::IndexedGeometry,
    render::tile_view_pattern::DEFAULT_TILE_SIZE,
};
use winit::event::{ElementState, MouseButton};

use crate::input::UpdateState;

pub struct QueryHandler {
    window_position: Option<Vector2<f64>>,
    clicking: bool,
}

impl QueryHandler {
    pub fn new() -> Self {
        Self {
            window_position: None,
            clicking: false,
        }
    }

    pub fn process_touch_start(&mut self) -> bool {
        self.clicking = true;
        true
    }

    pub fn process_touch_end(&mut self) -> bool {
        self.clicking = false;
        true
    }

    pub fn process_window_position(
        &mut self,
        window_position: &Vector2<f64>,
        _touch: bool,
    ) -> bool {
        self.window_position = Some(*window_position);
        true
    }

    pub fn process_mouse_key_press(&mut self, key: &MouseButton, state: &ElementState) -> bool {
        if *key != MouseButton::Left {
            return false;
        }

        if *state == ElementState::Pressed {
            self.clicking = true;
        } else {
            self.clicking = false;
        }
        true
    }
}

impl UpdateState for QueryHandler {
    fn update_state(
        &mut self,
        MapContext {
            view_state, world, ..
        }: &mut MapContext,
        _dt: Duration,
    ) {
        if self.clicking {
            if let Some(window_position) = self.window_position {
                let view_proj = view_state.view_projection();
                let inverted_view_proj = view_proj.invert();

                let z = view_state.zoom().zoom_level(DEFAULT_TILE_SIZE); // FIXME: can be wrong, if tiles of different z are visible
                let zoom = view_state.zoom();

                if let Some(coordinates) = view_state.window_to_world_at_ground(
                    &window_position,
                    &inverted_view_proj,
                    false,
                ) {
                    if let Some(geometries) = world
                        .tiles
                        .geometry_index
                        .query_point(
                            &WorldCoords {
                                x: coordinates.x,
                                y: coordinates.y,
                            },
                            z,
                            zoom,
                        )
                        .map(|geometries| {
                            geometries
                                .iter()
                                .cloned()
                                .cloned()
                                .collect::<Vec<IndexedGeometry<f64>>>()
                        })
                    {
                        log::info!(
                            "Clicked on geometry: {:?}",
                            geometries
                                .iter()
                                .map(|geometry| &geometry.properties)
                                .collect::<Vec<_>>()
                        );
                    } else {
                        log::info!("No geometry found.",);
                    }
                }
            }
            self.clicking = false;
        }
    }
}
