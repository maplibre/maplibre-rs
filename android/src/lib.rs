#![deny(unused_imports)]

use jni::{objects::JClass, JNIEnv};
use log::Level;
use maplibre::render::settings::WgpuSettings;
use maplibre_winit::{android_activity, run_headed_map, WinitMapWindowConfig};

#[cfg(not(any(no_pendantic_os_check, target_os = "android")))]
compile_error!("android works only on android.");

#[no_mangle]
pub fn android_main(app: android_activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );
    log::log!(Level::Info, "maplibre starting");
    run_headed_map::<String>(
        None,
        WinitMapWindowConfig::new("maplibre".to_string(), app),
        WgpuSettings {
            backends: Some(maplibre::render::settings::Backends::GL),
            ..WgpuSettings::default()
        },
    );
}

#[no_mangle]
pub extern "system" fn Java_org_maplibre_1rs_MapLibreRs_android_1main(
    _env: JNIEnv,
    _class: JClass,
) {
    log::log!(Level::Warn, "maplibre WOORKING");
}
