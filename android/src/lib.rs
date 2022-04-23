use std::ffi::CString;
use maplibre::window::FromWindow;
use maplibre::{MapBuilder, ScheduleMethod, TokioScheduleMethod};
pub use std::time::Instant;
use jni::JNIEnv;
use jni::objects::JClass;
use log::Level;

// TODO clippy
// #[cfg(not(target_os = "android"))]
// compile_error!("android works only on android.");

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn android_main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    MapBuilder::from_window("A fantastic window!")
        .with_schedule_method(ScheduleMethod::Tokio(TokioScheduleMethod::new()))
        .build()
        .run_sync();
}

#[no_mangle]
pub extern "system" fn Java_com_example_demo_MapLibre_android_1main(env: JNIEnv, class: JClass) {
    let tag = CString::new("maplibre").unwrap();
    let message = CString::new("maplibre WOORKING").unwrap();
    ndk_glue::android_log(Level::Warn, &tag,&message);

    //android_main();
}