use crate::{HttpClient, MapWindowConfig, ScheduleMethod};

pub trait Environment: 'static {
    type MapWindowConfig: MapWindowConfig;
    type ScheduleMethod: ScheduleMethod;
    type HttpClient: HttpClient;
}
