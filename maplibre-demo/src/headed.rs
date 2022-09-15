use maplibre::environment::DefaultTransferables;

use maplibre::platform::apc::TokioAsyncProcedureCall;
use maplibre::{
    platform::{http_client::ReqwestHttpClient, scheduler::TokioScheduler},
    MapBuilder,
};
use maplibre_winit::winit::{WinitEnvironment, WinitMapWindowConfig};

pub async fn run_headed() {
    MapBuilder::<WinitEnvironment<_, _, _, TokioAsyncProcedureCall>>::new()
        .with_map_window_config(WinitMapWindowConfig::new("maplibre".to_string()))
        .with_http_client(ReqwestHttpClient::new(None))
        .with_apc(TokioAsyncProcedureCall::new())
        .with_scheduler(TokioScheduler::new())
        .build()
        .initialize()
        .await
        .run()
}
