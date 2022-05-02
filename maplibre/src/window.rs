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

#[cfg(target_os = "android")]
/// On android we can not get the dimensions of the window initially. Therefore, we use a fallback
/// until the window is ready to deliver its correct bounds.
impl Default for WindowSize {
    fn default() -> Self {
        WindowSize {
            width: 100,
            height: 100,
        }
    }
}

pub type WindowFactory<W, E> = dyn FnOnce() -> (W, E);

pub trait FromWindow {
    fn from_window(title: &'static str) -> Self;
}

pub trait FromCanvas {
    fn from_canvas(dom_id: &'static str) -> Self;
}
