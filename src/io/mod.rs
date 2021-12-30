pub mod cache;
pub mod static_database;

#[derive(Clone, Copy, Debug)]
pub struct TileCoords {
    pub x: u32,
    pub y: u32,
    pub z: u8,
}

impl TileCoords {
    fn hash(&self) -> u32 {
        self.x + self.y + self.z as u32
    }
}

impl Into<TileCoords> for (u32, u32, u8) {
    fn into(self) -> TileCoords {
        TileCoords {
            x: self.0,
            y: self.1,
            z: self.2,
        }
    }
}
