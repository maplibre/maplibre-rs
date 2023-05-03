use std::time::Duration;

use maplibre::context::MapContext;

use super::UpdateState;

pub struct DebugHandler {
    inset_delta: f64,
}

impl UpdateState for DebugHandler {
    fn update_state(&mut self, MapContext { view_state, .. }: &mut MapContext, dt: Duration) {
        let dt = dt.as_secs_f64() * 10.0;

        let delta = self.inset_delta * dt;

        let mut edge_insets = *view_state.edge_insets();
        edge_insets.left += delta;
        view_state.set_edge_insets(edge_insets);
        self.inset_delta -= delta;
    }
}

impl DebugHandler {
    pub fn new() -> Self {
        Self { inset_delta: 0.0 }
    }

    pub fn process_key_press(
        &mut self,
        key: winit::event::VirtualKeyCode,
        state: winit::event::ElementState,
    ) -> bool {
        let amount = if state == winit::event::ElementState::Pressed {
            100.0
        } else {
            0.0
        };
        match key {
            winit::event::VirtualKeyCode::N => {
                self.inset_delta += amount;
                true
            }
            _ => false,
        }
    }
}
