//! Utilities for the window system.

use crate::{HTTPClient, MapSchedule, ScheduleMethod};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

/// Window with an optional [carte::window::WindowSize].
pub trait MapWindow {
    type EventLoop;
    type RawWindow;

    fn size(&self) -> WindowSize;

    fn inner(&self) -> &Self::RawWindow;
}

pub trait HasRawWindow {
    type HRWH: HasRawWindowHandle;

    fn raw_window(&self) -> &Self::HRWH;
}

impl<MW> HasRawWindow for MW
where
    MW: MapWindow,
    MW::RawWindow: HasRawWindowHandle,
{
    type HRWH = MW::RawWindow;

    fn raw_window(&self) -> &Self::HRWH {
        self.inner()
    }
}

pub trait MapWindowConfig: 'static {
    type MapWindow: MapWindow;

    fn create(&self) -> Self::MapWindow;
}

pub trait Runnable<MWC, SM, HC>
where
    MWC: MapWindowConfig,
    SM: ScheduleMethod,
    HC: HTTPClient,
{
    fn run(self, map_state: MapSchedule<MWC, SM, HC>, max_frames: Option<u64>);
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
