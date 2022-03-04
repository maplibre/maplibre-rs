//! Utils which are used internally

mod fps_meter;
pub mod math;

use crate::coords::WorldTileCoords;
pub use fps_meter::FPSMeter;

struct MinMaxBoundingBox {
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
    initialized: bool,
}

impl MinMaxBoundingBox {
    fn new() -> Self {
        Self {
            min_x: i32::MAX,
            min_y: i32::MAX,
            max_x: i32::MIN,
            max_y: i32::MIN,
            initialized: false,
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn update(&mut self, world_coords: &WorldTileCoords) {
        self.initialized = true;

        if world_coords.x < self.min_x {
            self.min_x = world_coords.x;
        }

        if world_coords.y < self.min_y {
            self.min_y = world_coords.y;
        }

        if world_coords.x > self.max_x {
            self.max_x = world_coords.x;
        }

        if world_coords.y > self.max_y {
            self.max_y = world_coords.y;
        }
    }
}
