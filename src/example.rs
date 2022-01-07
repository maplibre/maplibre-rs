use crate::io::cache::Cache;

pub const MUNICH_X: u32 = 17421;
pub const MUNICH_Y: u32 = 11360;
pub const MUNICH_Z: u8 = 15;

pub fn fetch_munich_tiles(cache: &Cache) {
    for x in 0..10 {
        for y in 0..10 {
            cache.fetch((MUNICH_X + x, MUNICH_Y + y, MUNICH_Z).into())
        }
    }
}
