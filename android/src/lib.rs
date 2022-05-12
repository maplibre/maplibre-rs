use jni::objects::JClass;
use jni::JNIEnv;
use log::Level;
use maplibre::platform::http_client::ReqwestHttpClient;
use maplibre::platform::schedule_method::TokioScheduleMethod;
use maplibre::window::FromWindow;
use maplibre::MapBuilder;
use std::ffi::CString;

#[cfg(not(target_os = "android"))]
compile_error!("android works only on android.");

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn android_main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    MapBuilder::from_window("A fantastic window!")
        .with_http_client(ReqwestHttpClient::new(None))
        .with_schedule_method(TokioScheduleMethod::new())
        .build()
        .run_sync();
}

#[no_mangle]
pub extern "system" fn Java_org_maplibre_1rs_MapLibreRs_android_1main(env: JNIEnv, class: JClass) {
    let tag = match CString::new("maplibre");
    let tag = match tag {
        Ok(tagg) => tagg,
        Err(e) => (e),
    };
    let message = CString::new("maplibre WOORKING");
    let message = match message {
        Ok(mess) => mess,
        Err(e) => (e),
    };
    ndk_glue::android_log(Level::Warn, &tag, &message);
}
