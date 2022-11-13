use crate::{
    environment::Environment,
    error::Error,
    map::Map,
    window::{HeadedMapWindow, MapWindowConfig},
};

pub trait EventLoopConfig {
    type EventType: 'static;
    type EventLoopProxy: EventLoopProxy<Self::EventType>;

    fn create_proxy() -> Self::EventLoopProxy;
}

pub trait EventLoopProxy<T: 'static> {
    fn send_event(&self, event: T) -> Result<(), Error>;
}

pub trait EventLoop<ET: 'static + PartialEq> {
    type EventLoopProxy: EventLoopProxy<ET>;

    fn run<E>(self, map: Map<E>, max_frames: Option<u64>)
    where
        E: Environment,
        <E::MapWindowConfig as MapWindowConfig>::MapWindow: HeadedMapWindow;

    fn create_proxy(&self) -> Self::EventLoopProxy;
}
