use jni::objects::JObject;
use jni::JNIEnv;
use maplibre::error::Error;
use maplibre::io::scheduler::ScheduleMethod;
use maplibre::io::source_client::HTTPClient;
use ndk::native_window::NativeWindow;
use raw_window_handle::{AndroidNdkHandle, RawWindowHandle};
use std::marker::PhantomData;
use std::thread::sleep;
use std::time::Duration;

use maplibre::map_state::MapState;
use maplibre::window::{MapWindow, MapWindowConfig, Runnable, WindowSize};

pub struct AndroidNativeWindow {
    window: NativeWindow,
}

pub struct AndroidMapWindowConfig<'a> {
    env: JNIEnv<'a>,
    surface: JObject<'a>,
}

unsafe impl raw_window_handle::HasRawWindowHandle for AndroidNativeWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        let mut handle = AndroidNdkHandle::empty();
        handle.a_native_window = unsafe { self.window.ptr().as_mut() as *mut _ as *mut _ };
        RawWindowHandle::AndroidNdk(handle)
    }
}

impl<'a> AndroidMapWindowConfig<'a> {
    pub fn new(env: JNIEnv<'a>, surface: JObject<'a>) -> Self {
        Self { env, surface }
    }
}

impl<'a> MapWindowConfig for AndroidMapWindowConfig<'a> {
    type MapWindow = AndroidMapWindow<'a>;
}

pub struct AndroidMapWindow<'a> {
    window: AndroidNativeWindow,
    phantom: PhantomData<&'a u32>,
}

impl AndroidMapWindow<'_> {
    pub fn take_event_loop(&mut self) -> Option<()> {
        Some(())
    }
}

impl<'a, MWC, SM, HC> Runnable<MWC, SM, HC> for AndroidMapWindow<'a>
where
    MWC: MapWindowConfig<MapWindow = AndroidMapWindow<'a>>,
    SM: ScheduleMethod,
    HC: HTTPClient,
{
    fn run(mut self, mut map_state: MapState<MWC, SM, HC>, max_frames: Option<u64>) {
        for i in 0..100 {
            map_state.update_and_redraw();
            sleep(Duration::from_millis(16))
        }

        match map_state.update_and_redraw() {
            Ok(_) => {}
            Err(Error::Render(e)) => {
                eprintln!("{}", e);
            }
            e => eprintln!("{:?}", e),
        };

        map_state.recreate_surface(&self);
        let size = self.size();
        map_state.resize(size.width(), size.height()); // FIXME: Resumed is also called when the app launches for the first time. Instead of first using a "fake" inner_size() in State::new we should initialize with a proper size from the beginning
        map_state.resume();
    }
}

impl<'a> MapWindow for AndroidMapWindow<'a> {
    type EventLoop = ();
    type Window = AndroidNativeWindow;
    type MapWindowConfig = AndroidMapWindowConfig<'a>;

    fn create(map_window_config: &Self::MapWindowConfig) -> Self {
        let window = unsafe {
            NativeWindow::from_surface(
                map_window_config.env.get_native_interface(),
                map_window_config.surface.into_inner(),
            )
        }
        .unwrap();

        Self {
            window: AndroidNativeWindow { window },
            phantom: Default::default(),
        }
    }

    fn size(&self) -> WindowSize {
        WindowSize::new(100, 100).unwrap()
    }

    fn inner(&self) -> &Self::Window {
        &self.window
    }
}
