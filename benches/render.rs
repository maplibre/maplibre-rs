use criterion::{criterion_group, criterion_main, Criterion};
use mapr::{MapBuilder, ScheduleMethod, TokioScheduleMethod};

fn render(c: &mut Criterion) {
    c.bench_function("render", |b| {
        b.iter(|| {
            env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

            MapBuilder::from_window("A fantastic window!")
                .with_schedule_method(ScheduleMethod::Tokio(TokioScheduleMethod::new(Some(
                    "/tmp/mapr_cache".to_string(),
                ))))
                .build()
                .run_sync_with_max_frames(Some(1000));
        })
    });
}

criterion_group!(benches, render);
criterion_main!(benches);
