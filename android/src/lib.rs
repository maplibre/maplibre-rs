#![deny(unused_imports)]

use jni::{objects::JClass, JNIEnv};
use log::Level;
use maplibre_winit::run_headed_map;

#[cfg(not(any(no_pendantic_os_check, target_os = "android")))]
compile_error!("android works only on android.");

#[no_mangle]
pub fn android_main() {
    android_logger::init_once(android_logger::Config::default());
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    // TODO: Maybe requires: Some(Backends::VULKAN)
    run_headed_map(None);
}

#[no_mangle]
pub extern "system" fn Java_org_maplibre_1rs_MapLibreRs_android_1main(
    _env: JNIEnv,
    _class: JClass,
) {
    log::log!(Level::Warn, "maplibre WOORKING");
}
