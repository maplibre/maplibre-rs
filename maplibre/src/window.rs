//! Utilities for the window system.

use std::num::NonZeroU32;

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

/// Window of a certain [`WindowSize`]. This can either be a proper window or a headless one.
pub trait MapWindow {
    fn size(&self) -> WindowSize;
}

/// Window which references a physical `RawWindow`. This is only implemented by headed windows and
/// not by headless windows.
pub trait HeadedMapWindow: MapWindow {
    type RawWindow: HasRawWindowHandle + HasRawDisplayHandle;

    fn raw(&self) -> &Self::RawWindow;

    // TODO: Can we avoid this?
    fn request_redraw(&self);

    fn id(&self) -> u64;
}

/// A configuration for a window which determines the corresponding implementation of a
/// [`MapWindow`] and is able to create it.
pub trait MapWindowConfig: 'static {
    type MapWindow: MapWindow;

    fn create(&self) -> Self::MapWindow;
}

/// Window size with a width and an height in pixels.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct WindowSize {
    width: NonZeroU32,
    height: NonZeroU32,
}

impl WindowSize {
    pub fn new(width: u32, height: u32) -> Option<Self> {
        Some(Self {
            width: NonZeroU32::new(width)?,
            height: NonZeroU32::new(height)?,
        })
    }

    pub fn width(&self) -> u32 {
        self.width.get()
    }

    pub fn width_non_zero(&self) -> NonZeroU32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height.get()
    }

    pub fn height_non_zero(&self) -> NonZeroU32 {
        self.height
    }
}
