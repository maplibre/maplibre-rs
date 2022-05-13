use crate::window::AndroidMapWindowConfig;
use jni::objects::{JClass, JObject};
use jni::JNIEnv;
use log::Level;
use maplibre::platform::http_client::ReqwestHttpClient;
use maplibre::platform::run_multithreaded;
use maplibre::platform::schedule_method::TokioScheduleMethod;
use maplibre::MapBuilder;
use maplibre_winit::winit::{WinitEventLoop, WinitMapWindow, WinitMapWindowConfig, WinitWindow};
use ndk::native_window::NativeWindow;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::unix::io::{FromRawFd, RawFd};
use std::thread;

mod window;

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
            .build()
            .initialize()
            .await
            .run()
    })
}

#[no_mangle]
pub extern "system" fn Java_org_maplibre_1rs_MapLibreRs_android_1main(
    env: JNIEnv,
    class: JClass,
    surface: JObject,
) {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let tag = CString::new("maplibre").unwrap();
    let message = CString::new("maplibre WOORKING").unwrap();
    ndk_glue::android_log(Level::Warn, &tag, &message);

    unsafe {
        let mut logpipe: [RawFd; 2] = Default::default();
        libc::pipe(logpipe.as_mut_ptr());
        libc::dup2(logpipe[1], libc::STDOUT_FILENO);
        libc::dup2(logpipe[1], libc::STDERR_FILENO);
        thread::spawn(move || {
            let tag = CStr::from_bytes_with_nul(b"MapLibreStderr\0").unwrap();
            let file = File::from_raw_fd(logpipe[0]);
            let mut reader = BufReader::new(file);
            let mut buffer = String::new();
            loop {
                buffer.clear();
                if let Ok(len) = reader.read_line(&mut buffer) {
                    if len == 0 {
                        break;
                    } else if let Ok(msg) = CString::new(buffer.clone()) {
                        ndk_glue::android_log(Level::Info, tag, &msg);
                    }
                }
            }
        });
    }

    run_multithreaded(async {
        MapBuilder::new()
            .with_map_window_config(AndroidMapWindowConfig::new(env, surface))
            .with_http_client(ReqwestHttpClient::new(None))
            .with_schedule_method(TokioScheduleMethod::new())
            .build()
            .initialize()
            .await
            .run()
    })
}
