//! Main (platform-specific) main loop which handles:
//! * Input (Mouse/Keyboard)
//! * Platform Events like suspend/resume
//! * Render a new frame

use winit::window::WindowBuilder;

use super::WinitEventLoop;
use super::WinitMapWindow;
use super::WinitWindow;

use maplibre::window::{MapWindow, WindowSize};

impl MapWindow for WinitMapWindow {
    type EventLoop = WinitEventLoop;
    type Window = WinitWindow;

    fn create() -> (Self, Self::EventLoop)
    where
        Self: Sized,
    {
        let event_loop = WinitEventLoop::new();
        let window = WindowBuilder::new()
            .with_title("title")
            .build(&event_loop)
            .unwrap();

        (Self { inner: window }, event_loop)
    }

    fn size(&self) -> WindowSize {
        let size = self.inner.inner_size();
        #[cfg(target_os = "android")]
        // On android we can not get the dimensions of the window initially. Therefore, we use a
        // fallback until the window is ready to deliver its correct bounds.
        let window_size = WindowSize::new(size.width, size.height).unwrap_or_default();

        #[cfg(not(target_os = "android"))]
        let window_size =
            WindowSize::new(size.width, size.height).expect("failed to get window dimensions.");
        window_size
    }

    fn inner(&self) -> &Self::Window {
        &self.inner
    }
}
