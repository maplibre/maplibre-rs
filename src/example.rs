use crate::io::worker_loop::WorkerLoop;

pub const MUNICH_X: u32 = 0;
pub const MUNICH_Y: u32 = 0;
pub const MUNICH_Z: u8 = 0;

pub fn fetch_munich_tiles(worker_loop: &mut WorkerLoop) {
    // This size matches the amount of tiles which are loaded on zoom 15 on FHD
    for x in 0..8 {
        for y in 0..5 {
            worker_loop.try_fetch((MUNICH_X + x, MUNICH_Y + y, MUNICH_Z).into())
        }
    }
}
