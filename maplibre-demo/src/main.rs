use maplibre::platform::http_client::ReqwestHttpClient;
use maplibre::platform::schedule_method::TokioScheduleMethod;
use maplibre::MapBuilder;
use maplibre_winit::winit::{WinitEventLoop, WinitMapWindow, WinitWindow};

#[cfg(feature = "enable-tracing")]
fn enable_tracing() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::Registry;

    let subscriber = Registry::default().with(tracing_tracy::TracyLayer::new());

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

fn run_in_window() {
    let builder: MapBuilder<WinitMapWindow, _, _, _> = MapBuilder::new();
    builder
        .with_http_client(ReqwestHttpClient::new(None))
        .with_schedule_method(TokioScheduleMethod::new())
        .build()
        .run_sync();
}

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    #[cfg(feature = "enable-tracing")]
    enable_tracing();

    run_in_window()
}
