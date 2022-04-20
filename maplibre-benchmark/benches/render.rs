use criterion::{criterion_group, criterion_main, Criterion};
use maplibre::window::FromWindow;
use maplibre::{MapBuilder, ScheduleMethod, TokioScheduleMethod};

fn render(c: &mut Criterion) {
    c.bench_function("render", |b| {
        b.iter(|| {
            MapBuilder::from_window("A fantastic window!")
                .with_schedule_method(ScheduleMethod::Tokio(TokioScheduleMethod::new()))
                .build()
                .run_sync_with_max_frames(1000);
        })
    });
}

criterion_group!(benches, render);
criterion_main!(benches);
