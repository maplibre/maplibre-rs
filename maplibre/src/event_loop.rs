use crate::environment::Environment;
use crate::map::Map;
use crate::window::{HeadedMapWindow, MapWindowConfig};

pub trait EventLoopConfig {
    type EventType: 'static;
    type EventLoopProxy: EventLoopProxy<Self::EventType>;

    fn create_proxy() -> Self::EventLoopProxy;
}

pub trait EventLoopProxy<T: 'static> {
    fn send_event(&self, event: T);
}

pub trait EventLoop<T: 'static> {
    type EventLoopProxy: EventLoopProxy<T>;

    fn run<E>(
        self,
        window: <E::MapWindowConfig as MapWindowConfig>::MapWindow,
        map: Map<E>,
        max_frames: Option<u64>,
    ) where
        E: Environment,
        <E::MapWindowConfig as MapWindowConfig>::MapWindow: HeadedMapWindow;

    fn create_proxy(&self) -> Self::EventLoopProxy;
}
