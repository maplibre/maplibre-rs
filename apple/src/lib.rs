use maplibre::window::FromWindow;
use maplibre::{MapBuilder, ScheduleMethod, TokioScheduleMethod};
pub use std::time::Instant;

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
compile_error!("apple works only on macOS and iOS.");

#[no_mangle]
pub fn maplibre_apple_main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    MapBuilder::from_window("A fantastic window!")
        .with_schedule_method(ScheduleMethod::Tokio(TokioScheduleMethod::new()))
        .build()
        .run_sync();
}
