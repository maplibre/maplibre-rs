//! Utilities for the window system.

use crate::{HTTPClient, MapState, ScheduleMethod};

/// Window with an optional [carte::window::WindowSize].
pub trait MapWindow {
    type EventLoop;
    type Window: raw_window_handle::HasRawWindowHandle;
    type MapWindowConfig: MapWindowConfig<MapWindow = Self>;

    fn create(map_window_config: &Self::MapWindowConfig) -> Self;

    fn size(&self) -> WindowSize;

    fn inner(&self) -> &Self::Window;
}

pub trait MapWindowConfig {
    type MapWindow: MapWindow<MapWindowConfig = Self>;
}

pub trait Runnable<MWC, SM, HC>
where
    MWC: MapWindowConfig,
    SM: ScheduleMethod,
    HC: HTTPClient,
{
    fn run(self, map_state: MapState<MWC, SM, HC>, max_frames: Option<u64>);
}

/// Window size with a width and an height in pixels.
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
