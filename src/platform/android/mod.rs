use crate::io::scheduler::ScheduleMethod;
use crate::platform::schedule_method::TokioScheduleMethod;
use crate::MapBuilder;
pub use std::time::Instant;

pub const COLOR_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    MapBuilder::from_window("A fantastic window!")
        .with_schedule_method(ScheduleMethod::Tokio(TokioScheduleMethod::new(Some(
            "/tmp/mapr_cache".to_string(),
        ))))
        .build()
        .run_sync();
}
