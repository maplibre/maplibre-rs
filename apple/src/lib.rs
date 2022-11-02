use maplibre::{
    io::apc::SchedulerAsyncProcedureCall,
    platform::{http_client::ReqwestHttpClient, run_multithreaded, scheduler::TokioScheduler},
};
use maplibre_winit::{
    run_headed_map, WinitEnvironment, WinitEventLoop, WinitMapWindow, WinitMapWindowConfig,
};

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
compile_error!("apple works only on macOS and iOS.");

#[no_mangle]
pub fn maplibre_apple_main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    run_headed_map(None);
}
