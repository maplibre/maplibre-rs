#![deny(unused_imports)]

use std::ffi::CString;

use jni::{objects::JClass, JNIEnv};
use log::Level;
use maplibre_winit::run_headed_map;

#[cfg(not(any(no_pendantic_os_check, target_os = "android")))]
compile_error!("android works only on android.");

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn android_main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    // TODO: Maybe requires: Some(Backends::VULKAN)
    run_headed_map(None);
}

#[no_mangle]
pub extern "system" fn Java_org_maplibre_1rs_MapLibreRs_android_1main(
    _env: JNIEnv,
    _class: JClass,
) {
    let tag = CString::new("maplibre").unwrap();
    let message = CString::new("maplibre WOORKING").unwrap();
    ndk_glue::android_log(Level::Warn, &tag, &message);
}
