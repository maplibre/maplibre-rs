use crate::{HttpClient, MapWindowConfig, Scheduler};

pub trait Environment: 'static {
    type MapWindowConfig: MapWindowConfig;
    type Scheduler: Scheduler;
    type HttpClient: HttpClient;
}
