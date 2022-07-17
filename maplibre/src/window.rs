//! Utilities for the window system.

use raw_window_handle::HasRawWindowHandle;

use crate::{HttpClient, InteractiveMapSchedule, ScheduleMethod};

/// Window of a certain [`WindowSize`]. This can either be a proper window or a headless one.
pub trait MapWindow {
    fn size(&self) -> WindowSize;
}

/// Window which references a physical `RawWindow`. This is only implemented by headed windows and
/// not by headless windows.
pub trait HeadedMapWindow: MapWindow {
    type RawWindow: HasRawWindowHandle;

    fn inner(&self) -> &Self::RawWindow;
}

/// A configuration for a window which determines the corresponding implementation of a
/// [`MapWindow`] and is able to create it.
pub trait MapWindowConfig: 'static {
    type MapWindow: MapWindow;

    fn create(&self) -> Self::MapWindow;
}

/// The event loop is responsible for processing events and propagating them to the map renderer.
/// Only non-headless windows use an [`EventLoop`].
pub trait EventLoop<MWC, SM, HC>
where
    MWC: MapWindowConfig,
    SM: ScheduleMethod,
    HC: HttpClient,
{
    fn run(self, map_schedule: InteractiveMapSchedule<MWC, SM, HC>, max_frames: Option<u64>);
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
