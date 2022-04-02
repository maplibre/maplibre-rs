use mapr::{MapBuilder, ScheduleMethod, TokioScheduleMethod};

#[cfg(feature = "enable-tracing")]
fn enable_tracing() {
    use tracing::{error, span};
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::Registry;

    let subscriber = Registry::default().with(tracing_tracy::TracyLayer::new());

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    #[cfg(feature = "enable-tracing")]
    enable_tracing();

    MapBuilder::from_window("A fantastic window!")
        .with_schedule_method(ScheduleMethod::Tokio(TokioScheduleMethod::new(Some(
            "/tmp/mapr_cache".to_string(),
        ))))
        .build()
        .run_sync();
}
