use maplibre::window::FromWindow;
use maplibre::{MapBuilder, ScheduleMethod, TokioScheduleMethod};
pub use std::time::Instant;

// TODO clippy
// #[cfg(not(target_os = "android"))]
// compile_error!("maplibre-android works only on android.");

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    MapBuilder::from_window("A fantastic window!")
        .with_schedule_method(ScheduleMethod::Tokio(TokioScheduleMethod::new()))
        .build()
        .run_sync();
}
