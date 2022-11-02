use std::time::Duration;

use cgmath::Vector2;
use maplibre::world::ViewState;
use winit::event::{ElementState, MouseButton};

use crate::input::UpdateState;

pub struct QueryHandler {
    window_position: Option<Vector2<f64>>,
    clicking: bool,
}

/*impl UpdateState for QueryHandler {

}*/

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
    fn update_state(&mut self, state: &mut ViewState, _dt: Duration) {
        if self.clicking {
            if let Some(window_position) = self.window_position {
                let view_proj = state.view_projection();
                let inverted_view_proj = view_proj.invert();

                let _z = state.visible_level(); // FIXME: can be wrong, if tiles of different z are visible
                let _zoom = state.zoom();

                if let Some(_coordinates) = state.camera().window_to_world_at_ground(
                    &window_position,
                    &inverted_view_proj,
                    false,
                ) {
                    // TODO reenable
                    /*state
                    .scheduler()
                    .schedule(state.scheduler(), move |thread_local| async move {
                        if let Some(geometries) = thread_local.query_point(
                            &WorldCoords {
                                x: coordinates.x,
                                y: coordinates.y,
                            },
                            z,
                            zoom,
                        ) {
                            log::info!(
                                "{:?}",
                                geometries
                                    .iter()
                                    .map(|geometry| &geometry.properties)
                                    .collect::<Vec<_>>()
                            );
                        }
                    })
                    .unwrap();*/
                }
            }
            self.clicking = false;
        }
    }
}
