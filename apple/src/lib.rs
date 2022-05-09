use maplibre::platform::http_client::ReqwestHttpClient;
use maplibre::platform::schedule_method::TokioScheduleMethod;
use maplibre::MapBuilder;
use maplibre_winit::winit::{WinitEventLoop, WinitMapWindow, WinitWindow};

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
compile_error!("apple works only on macOS and iOS.");

#[no_mangle]
pub fn maplibre_apple_main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let builder: MapBuilder<WinitMapWindow, _, _, _> = MapBuilder::new();
    builder
        .with_http_client(ReqwestHttpClient::new(None))
        .with_schedule_method(TokioScheduleMethod::new())
        .build()
        .run_sync();
}
