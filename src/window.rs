#[derive(Clone, Copy)]
pub struct WindowSize {
    width: u32,
    height: u32,
}

impl WindowSize {
    pub fn new(width: u32, height: u32) -> Option<Self> {
        if width <= 0 || height <= 0 {
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

pub type WindowFactory<W, E> = dyn FnOnce() -> (W, WindowSize, E);

pub trait FromWindow {
    fn from_window(title: &'static str) -> Self;
}

pub trait FromCanvas {
    fn from_canvas(dom_id: &'static str) -> Self;
}
