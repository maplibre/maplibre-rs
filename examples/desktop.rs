use mapr::{MapBuilder, ScheduleMethod, TokioScheduleMethod};

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    MapBuilder::from_window("A fantastic window!")
        .with_schedule_method(ScheduleMethod::Tokio(TokioScheduleMethod::new(Some(
            "/tmp/mapr_cache".to_string(),
        ))))
        .build()
        .run_sync();
}
