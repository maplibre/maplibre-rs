use std::fmt::{Display, Formatter};

use crate::{
    environment::Environment,
    map::Map,
    window::{HeadedMapWindow, MapWindowConfig},
};

pub trait EventLoopConfig {
    type EventType: 'static;
    type EventLoopProxy: EventLoopProxy<Self::EventType>;

    fn create_proxy() -> Self::EventLoopProxy;
}

/// When sending events to an event loop errors can occur.
#[derive(Debug)]
pub enum SendEventError {
    /// The event loop was already closed
    Closed,
}

impl Display for SendEventError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for SendEventError {}

pub trait EventLoopProxy<T: 'static> {
    fn send_event(&self, event: T) -> Result<(), SendEventError>;
}

pub trait EventLoop<ET: 'static + PartialEq> {
    type EventLoopProxy: EventLoopProxy<ET>;

    fn run<E>(self, map: Map<E>, max_frames: Option<u64>)
    where
        E: Environment,
        <E::MapWindowConfig as MapWindowConfig>::MapWindow: HeadedMapWindow;

    fn create_proxy(&self) -> Self::EventLoopProxy;
}
