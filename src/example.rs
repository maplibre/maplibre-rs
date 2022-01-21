use crate::io::worker_loop::WorkerLoop;

pub const MUNICH_X: u32 = 17425;
pub const MUNICH_Y: u32 = 11365;
pub const MUNICH_Z: u8 = 15;

pub fn fetch_munich_tiles(worker_loop: &mut WorkerLoop) {
    // This size matches the amount of tiles which are loaded on zoom 15 on FHD
    for x in 0..8 {
        for y in 0..5 {
            worker_loop.fetch((MUNICH_X + x, MUNICH_Y + y, MUNICH_Z).into())
        }
    }
}
