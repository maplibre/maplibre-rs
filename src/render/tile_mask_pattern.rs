use crate::coords::WorldTileCoords;

/// The tile mask pattern assigns each tile a value which can be used for stencil testing.
/// The pattern can be reviewed [here](https://maxammann.org/mapr/docs/stencil-masking.html).
pub struct TileMaskPattern {}

impl TileMaskPattern {
    pub fn new() -> Self {
        Self {}
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
}
