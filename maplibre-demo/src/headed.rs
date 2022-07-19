use maplibre::{
    platform::{http_client::ReqwestHttpClient, schedule_method::TokioScheduleMethod},
    MapBuilder,
};
use maplibre_winit::winit::WinitMapWindowConfig;

pub async fn run_headed() {
    MapBuilder::new()
        .with_map_window_config(WinitMapWindowConfig::new("maplibre".to_string()))
        .with_http_client(ReqwestHttpClient::new(None))
        .with_schedule_method(TokioScheduleMethod::new())
        .build()
        .initialize()
        .await
        .run()
}
