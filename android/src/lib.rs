use std::ffi::CString;

use jni::{objects::JClass, JNIEnv};
use log::Level;
use maplibre::{
    platform::{
        http_client::ReqwestHttpClient, run_multithreaded, schedule_method::TokioScheduleMethod,
    },
    render::settings::{Backends, WgpuSettings},
    MapBuilder,
};
use maplibre_winit::winit::WinitMapWindowConfig;

#[cfg(not(target_os = "android"))]
compile_error!("android works only on android.");

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn android_main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    run_multithreaded(async {
        MapBuilder::new()
            .with_map_window_config(WinitMapWindowConfig::new("maplibre android".to_string()))
            .with_http_client(ReqwestHttpClient::new(None))
            .with_schedule_method(TokioScheduleMethod::new())
            .with_wgpu_settings(WgpuSettings {
                backends: Some(Backends::VULKAN),
                ..WgpuSettings::default()
            })
            .build()
            .initialize()
            .await
            .run()
    })
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
