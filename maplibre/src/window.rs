//! Utilities for the window system.

use crate::{HttpClient, MapSchedule, ScheduleMethod};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

/// Window with a [carte::window::WindowSize].
pub trait MapWindow {
    fn size(&self) -> WindowSize;
}

pub trait HeadedMapWindow: MapWindow {
    type RawWindow: HasRawWindowHandle;

    fn inner(&self) -> &Self::RawWindow;
}

pub trait MapWindowConfig: 'static {
    type MapWindow: MapWindow;

    fn create(&self) -> Self::MapWindow;
}

pub trait EventLoop<MWC, SM, HC>
where
    MWC: MapWindowConfig,
    SM: ScheduleMethod,
    HC: HttpClient,
{
    fn run(self, map_schedule: MapSchedule<MWC, SM, HC>, max_frames: Option<u64>);
}

/// Window size with a width and an height in pixels.
#[derive(Clone, Copy, Eq, PartialEq)]
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
