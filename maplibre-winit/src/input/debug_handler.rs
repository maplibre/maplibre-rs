use std::time::Duration;

use maplibre::context::MapContext;
use winit::keyboard::Key;

use super::UpdateState;

#[derive(Default)]
pub struct DebugHandler {
    top_delta: f64,
    bottom_delta: f64,
    left_delta: f64,
    right_delta: f64,
}

impl UpdateState for DebugHandler {
    fn update_state(&mut self, MapContext { view_state, .. }: &mut MapContext, dt: Duration) {
        let dt = dt.as_secs_f64() * 10.0;

        let top_delta = self.top_delta * dt;
        let bottom_delta = self.bottom_delta * dt;
        let left_delta = self.left_delta * dt;
        let right_delta = self.right_delta * dt;

        let mut edge_insets = *view_state.edge_insets();
        edge_insets.top += top_delta;
        edge_insets.bottom += bottom_delta;
        edge_insets.left += left_delta;
        edge_insets.right += right_delta;
        view_state.set_edge_insets(edge_insets);
        self.top_delta -= top_delta;
        self.bottom_delta -= bottom_delta;
        self.right_delta -= right_delta;
        self.left_delta -= left_delta;
    }
}

impl DebugHandler {
    pub fn process_key_press(&mut self, key: &Key, state: winit::event::ElementState) -> bool {
        let amount = if state == winit::event::ElementState::Pressed {
            100.0
        } else {
            0.0
        };

        match key.as_ref() {
            Key::Character("v") => {
                self.left_delta += amount;
                true
            }
            Key::Character("b") => {
                self.top_delta += amount;
                true
            }
            Key::Character("n") => {
                self.bottom_delta += amount;
                true
            }
            Key::Character("m") => {
                self.right_delta += amount;
                true
            }
            _ => false,
        }
    }
}
