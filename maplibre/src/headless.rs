use crate::render::render_state::HeadlessMapSurface;
use crate::{
    HTTPClient, MapBuilder, MapState, MapSurface, MapWindow, Runnable, ScheduleMethod, WindowSize,
};
use wgpu::Instance;

pub struct HeadlessWindow {
    size: WindowSize,
}

impl MapWindow<()> for HeadlessWindow {
    fn create(_instance: &Instance) -> (Self, MapSurface, WindowSize, ()) {
        let size = WindowSize::new(100, 100).unwrap();
        (
            HeadlessWindow { size },
            MapSurface::Headless(HeadlessMapSurface::initialize(size)),
            size,
            (),
        )
    }

    fn size(&self) -> Option<WindowSize> {
        Some(self.size)
    }
}

impl<SM, HC> Runnable<()> for MapState<HeadlessWindow, (), SM, HC>
where
    SM: ScheduleMethod,
    HC: HTTPClient,
{
    fn run(mut self, event_loop: (), max_frames: Option<u64>) {
        match self.update_and_redraw() {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost) => {
                log::error!("Surface Lost");
            }
            // The system is out of memory, we should probably quit
            Err(wgpu::SurfaceError::OutOfMemory) => {
                log::error!("Out of Memory");
            }
            // All other errors (Outdated, Timeout) should be resolved by the next frame
            Err(e) => eprintln!("{:?}", e),
        };
    }
}

/*impl<SM, HC> FromWindow for MapBuilder<HeadlessWindow, (), SM, HC>
where
    SM: ScheduleMethod,
    HC: HTTPClient,
{
    fn from_window(title: &'static str) -> Self {

    }
}
*/
