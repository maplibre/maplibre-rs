use maplibre::io::transferables::DefaultTransferables;
use maplibre::{
    io::apc::SchedulerAsyncProcedureCall,
    platform::{http_client::ReqwestHttpClient, scheduler::TokioScheduler},
    MapBuilder,
};
use maplibre_winit::winit::{WinitEnvironment, WinitMapWindowConfig};

pub async fn run_headed() {
    let client = ReqwestHttpClient::new(None);
    MapBuilder::<WinitEnvironment<_, _, _, SchedulerAsyncProcedureCall<_, _>>>::new()
        .with_map_window_config(WinitMapWindowConfig::new("maplibre".to_string()))
        .with_http_client(client.clone())
        .with_apc(SchedulerAsyncProcedureCall::new(
            client,
            TokioScheduler::new(),
        ))
        .with_scheduler(TokioScheduler::new())
        .build()
        .initialize()
        .await
        .run()
}
