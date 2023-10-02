#![deny(unused_imports)]

use jni::{objects::JClass, JNIEnv};
use log::Level;
use maplibre_winit::run_headed_map;

#[cfg(not(any(no_pendantic_os_check, target_os = "android")))]
compile_error!("android works only on android.");

#[no_mangle]
pub fn android_main(app: android_activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );
    log::log!(Level::Warn, "maplibre warn");
    log::log!(Level::Info, "maplibre info");
    // TODO: Maybe requires: Some(Backends::VULKAN)
    run_headed_map(None, app);
}

#[no_mangle]
pub extern "system" fn Java_org_maplibre_1rs_MapLibreRs_android_1main(
    _env: JNIEnv,
    _class: JClass,
) {
    log::log!(Level::Warn, "maplibre WOORKING");
}
