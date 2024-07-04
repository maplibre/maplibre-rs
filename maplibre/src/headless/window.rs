use crate::window::{MapWindow, MapWindowConfig, PhysicalSize, WindowCreateError};

#[derive(Clone)]
pub struct HeadlessMapWindowConfig {
    size: PhysicalSize,
}

impl HeadlessMapWindowConfig {
    pub fn new(size: PhysicalSize) -> Self {
        Self { size }
    }
}

impl MapWindowConfig for HeadlessMapWindowConfig {
    type MapWindow = HeadlessMapWindow;

    fn create(&self) -> Result<Self::MapWindow, WindowCreateError> {
        Ok(Self::MapWindow { size: self.size })
    }
}

pub struct HeadlessMapWindow {
    size: PhysicalSize,
}

impl MapWindow for HeadlessMapWindow {
    fn size(&self) -> PhysicalSize {
        self.size
    }
}
