use jni::objects::JObject;
use jni::JNIEnv;
use maplibre::error::Error;
use maplibre::io::scheduler::ScheduleMethod;
use maplibre::io::source_client::HttpClient;
use maplibre::map_schedule::InteractiveMapSchedule;
use ndk::native_window::NativeWindow;
use raw_window_handle::{AndroidNdkHandle, RawWindowHandle};
use std::marker::PhantomData;
use std::thread::sleep;
use std::time::Duration;

use maplibre::window::{EventLoop, HeadedMapWindow, MapWindow, MapWindowConfig, WindowSize};

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

    fn create(&self) -> Self::MapWindow {
        let window = unsafe {
            NativeWindow::from_surface(self.env.get_native_interface(), self.surface.into_inner())
        }
        .unwrap();

        Self::MapWindow {
            window: AndroidNativeWindow { window },
            phantom: Default::default(),
        }
    }
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

impl<'a, MWC, SM, HC> EventLoop<MWC, SM, HC> for AndroidMapWindow<'a>
where
    MWC: MapWindowConfig<MapWindow = AndroidMapWindow<'a>>,
    SM: ScheduleMethod,
    HC: HttpClient,
{
    fn run(
        mut self,
        mut map_schedule: InteractiveMapSchedule<MWC, SM, HC>,
        max_frames: Option<u64>,
    ) {
        for i in 0..100 {
            map_schedule.update_and_redraw();
            sleep(Duration::from_millis(16))
        }

        match map_schedule.update_and_redraw() {
            Ok(_) => {}
            Err(Error::Render(e)) => {
                eprintln!("{}", e);
            }
            e => eprintln!("{:?}", e),
        };

        let size = self.size();
        map_schedule.resize(size.width(), size.height()); // FIXME: Resumed is also called when the app launches for the first time. Instead of first using a "fake" inner_size() in State::new we should initialize with a proper size from the beginning
        map_schedule.resume(&self);
    }
}

impl<'a> MapWindow for AndroidMapWindow<'a> {
    fn size(&self) -> WindowSize {
        WindowSize::new(100, 100).unwrap()
    }
}

impl<'a> HeadedMapWindow for AndroidMapWindow<'a> {
    type RawWindow = AndroidNativeWindow;

    fn inner(&self) -> &Self::RawWindow {
        &self.window
    }
}
