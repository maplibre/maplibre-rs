use crate::io::cache::Cache;

pub const MUNICH_OFFSET_X: u32 = 2178;
pub const MUNICH_OFFSET_Y: u32 = 1421;

pub fn fetch_munich_tiles(cache: &Cache) {
    for x in 0..6 {
        for y in 0..6 {
            cache.fetch((MUNICH_OFFSET_X + x, MUNICH_OFFSET_Y + y, 12).into())
        }
    }
}
