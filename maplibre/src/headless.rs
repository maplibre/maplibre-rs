use crate::render::render_state::{Frame, HeadlessMapSurface};
use crate::{
    HTTPClient, MapBuilder, MapState, MapSurface, MapWindow, Runnable, ScheduleMethod, WindowSize,
};
use std::fs::File;
use std::thread::sleep;
use std::time::Duration;
use tokio::task;
use wgpu::Instance;

pub struct HeadlessWindow {
    size: WindowSize,
}

impl MapWindow<()> for HeadlessWindow {
    fn create(_instance: &Instance) -> (Self, MapSurface, WindowSize, ()) {
        let size = WindowSize::new(1920, 1080).unwrap();
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
        for i in 0..100 {
            self.update_and_redraw();
            sleep(Duration::from_millis(16))
        }

        match self.update_and_redraw() {
            Ok(frame) => match frame {
                Frame::Window(_) => {}
                Frame::Headless(headless) => {
                    let device = self.render_state().device.clone();
                    task::spawn(
                        async move { headless.create_png("test.png", device.as_ref()).await },
                    );
                }
            },
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
