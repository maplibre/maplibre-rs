use crate::{HTTPClient, MapState, ScheduleMethod};

pub trait MapWindow {
    type EventLoop;
    type Window: raw_window_handle::HasRawWindowHandle;

    fn create() -> Self;

    fn size(&self) -> WindowSize;

    fn inner(&self) -> &Self::Window;
}

pub trait MapWindowConfig {
    type WindowMap: MapWindow;
}

pub trait RunnableWindowMap<SM, HC>: MapWindow + Runnable<SM, HC>
where
    SM: ScheduleMethod,
    HC: HTTPClient,
{
}

pub trait Runnable<SM, HC>
where
    SM: ScheduleMethod,
    HC: HTTPClient,
{
    fn run(self, map_state: MapState<SM, HC>, max_frames: Option<u64>);
}

#[derive(Clone, Copy)]
pub struct WindowSize {
    width: u32,
    height: u32,
}

impl WindowSize {
    pub fn new(width: u32, height: u32) -> Option<Self> {
        if width == 0 || height == 0 {
            return None;
        }

        Some(Self { width, height })
    }

    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
}
