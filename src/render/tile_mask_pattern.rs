use crate::coords::WorldTileCoords;
use crate::render::shaders::ShaderTileMaskInstance;

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

pub struct TileMaskPattern {
    bounding_box: MinMaxBoundingBox,
    pattern: Vec<ShaderTileMaskInstance>,
}

/// Implementation of a masking algorithm using a stencil buffer. The layout of the
/// buffer can be reviewed [here](https://maxammann.org/mapr/docs/stencil-masking.html).
///
impl TileMaskPattern {
    pub fn new() -> Self {
        Self {
            bounding_box: MinMaxBoundingBox::new(),
            pattern: vec![],
        }
    }

    pub fn update_bounds(&mut self, world_coords: &WorldTileCoords) {
        self.bounding_box.update(world_coords)
    }

    pub fn as_slice(&self) -> &[ShaderTileMaskInstance] {
        self.pattern.as_slice()
    }

    pub fn instances(&self) -> u32 {
        self.pattern.len() as u32
    }

    fn vertical(&mut self, dx: i32, dy: i32, anchor_x: f32, anchor_y: f32, extent: f32) {
        for i in 0..(dx.abs() / 2 + 1) {
            self.pattern.push(ShaderTileMaskInstance::new(
                [anchor_x + ((i * 2) + 1) as f32 * extent, anchor_y],
                1.0,
                dy as f32,
                [0.0, 1.0, 0.0, 1.0],
            ));
        }
    }

    fn horizontal(&mut self, dx: i32, dy: i32, anchor_x: f32, anchor_y: f32, extent: f32) {
        for i in 0..(dy.abs() / 2 + 1) {
            self.pattern.push(ShaderTileMaskInstance::new(
                [anchor_x, anchor_y + (i * 2) as f32 * extent],
                dx as f32,
                1.0,
                [0.0, 0.0, 1.0, 1.0],
            ));
        }
    }

    pub fn stencil_reference_value(&self, world_coords: &WorldTileCoords) -> u8 {
        match (world_coords.x, world_coords.y) {
            (x, y) if x % 2 == 0 && y % 2 == 0 => 2,
            (x, y) if x % 2 == 0 && y % 2 != 0 => 1,
            (x, y) if x % 2 != 0 && y % 2 == 0 => 4,
            (x, y) if x % 2 != 0 && y % 2 != 0 => 3,
            _ => unreachable!(),
        }
    }

    pub fn update_pattern(&mut self, z: u8, extent: f32) {
        if !self.bounding_box.is_initialized() {
            // Happens if `update_bounds` hasn't been called so far
            return;
        }

        self.pattern.clear();

        let start: WorldTileCoords = (self.bounding_box.min_x, self.bounding_box.min_y, z).into(); // upper left corner
        let end: WorldTileCoords = (self.bounding_box.max_x, self.bounding_box.max_y, z).into(); // lower right corner

        let aligned_start = start.into_aligned().upper_left();
        let aligned_end = end.into_aligned().lower_right();

        let start_world = aligned_start.into_world(extent);

        let dy = aligned_end.y - aligned_start.y + 1;
        let dx = aligned_end.x - aligned_start.x + 1;

        let anchor_x = start_world.x;
        let anchor_y = start_world.y;
        // red step
        self.pattern.push(ShaderTileMaskInstance::new(
            [anchor_x, anchor_y],
            dx as f32,
            dy as f32,
            [1.0, 0.0, 0.0, 1.0],
        ));

        // green step
        self.vertical(dx, dy, anchor_x, anchor_y, extent);

        // blue step
        self.horizontal(dx, dy, anchor_x, anchor_y, extent);

        // violet step
        self.vertical(dx, dy, anchor_x, anchor_y, extent);
    }
}
