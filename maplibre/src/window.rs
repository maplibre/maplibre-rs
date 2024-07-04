//! Utilities for the window system.

use std::num::NonZeroU32;

use thiserror::Error;
use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};

/// Window of a certain [`PhysicalSize`]. This can either be a proper window or a headless one.
pub trait MapWindow {
    fn size(&self) -> PhysicalSize;
}

/// Window which references a physical `RawWindow`. This is only implemented by headed windows and
/// not by headless windows.
pub trait HeadedMapWindow: MapWindow {
    type WindowHandle: HasWindowHandle + HasDisplayHandle + Sync;

    fn handle(&self) -> &Self::WindowHandle;

    // TODO: Can we avoid this?
    fn request_redraw(&self);

    fn scale_factor(&self) -> f64;

    fn id(&self) -> u64;
}

#[derive(Error, Debug)]
pub enum WindowCreateError {
    #[error("unable to create event loop")]
    EventLoop,
    #[error("unable to create window")]
    Window,
}

/// A configuration for a window which determines the corresponding implementation of a
/// [`MapWindow`] and is able to create it.
pub trait MapWindowConfig: 'static + Clone {
    type MapWindow: MapWindow;

    fn create(&self) -> Result<Self::MapWindow, WindowCreateError>;
}

/// Window size with a width and an height in pixels.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PhysicalSize {
    width: NonZeroU32,
    height: NonZeroU32,
}

impl PhysicalSize {
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct LogicalSize {
    width: NonZeroU32,
    height: NonZeroU32,
}

impl LogicalSize {
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

impl PhysicalSize {
    pub fn to_logical(&self, scale_factor: f64) -> LogicalSize {
        let width = self.width.get() as f64 / scale_factor;
        let height = self.height.get() as f64 / scale_factor;
        LogicalSize {
            width: NonZeroU32::new(width as u32).expect("impossible to reach"),
            height: NonZeroU32::new(height as u32).expect("impossible to reach"),
        }
    }
}
