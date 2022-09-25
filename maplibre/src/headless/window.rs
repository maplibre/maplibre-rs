use crate::window::{MapWindow, MapWindowConfig, WindowSize};

pub struct HeadlessMapWindowConfig {
    size: WindowSize,
}

impl HeadlessMapWindowConfig {
    pub fn new(size: WindowSize) -> Self {
        Self { size }
    }
}

impl MapWindowConfig for HeadlessMapWindowConfig {
    type MapWindow = HeadlessMapWindow;

    fn create(&self) -> Self::MapWindow {
        Self::MapWindow { size: self.size }
    }
}

pub struct HeadlessMapWindow {
    size: WindowSize,
}

impl MapWindow for HeadlessMapWindow {
    fn size(&self) -> WindowSize {
        self.size
    }
}
