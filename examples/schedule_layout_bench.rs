//! Local release-mode schedule layout probe; timings are observations, not CI thresholds.
//! Run with: cargo run --release --features dates --example schedule_layout_bench
use sapid::prelude::*;
use std::{
    hint::black_box,
    time::{Duration, Instant},
};

const RUNS: usize = 30;

fn date(year: u16, month: u8, day: u8) -> CivilDate {
    CivilDate::new(year, month, day).unwrap()
}
fn dt(day: CivilDate, hour: u8, minute: u8) -> LocalDateTime {
    LocalDateTime::new(day, LocalTime::new(hour, minute, 0).unwrap())
}
fn key(value: impl Into<String>) -> ScheduleKey {
    ScheduleKey::new(value).unwrap()
}
fn fixture() -> ResourceScheduleData {
    let start = date(2026, 6, 1);
    let resources =
        ScheduleResources::new((0..8).map(|i| {
            ScheduleResource::new(key(format!("room-{i}")), format!("Room {i}")).unwrap()
        }))
        .unwrap();
    let yearly = ScheduleRecurrence::new(
        RecurrenceFrequency::Yearly {
            missing: MissingScheduleDay::Clamp,
        },
        1,
        RecurrenceEnd::Never,
    )
    .unwrap();
    let mut events = Vec::with_capacity(512);
    // Fourteen overlapping long annual events per resource exercise the bounded
    // duration lookback. They are timed events (all_day remains false).
    for i in 0..112 {
        events.push(
            ScheduleEvent::new(
                key(format!("long-{i}")),
                "Long annual event",
                ScheduleInterval::new(dt(date(2024, 1, 1), 0, 0), dt(date(2024, 12, 31), 0, 0))
                    .unwrap(),
            )
            .unwrap()
            .resource(key(format!("room-{}", i % 8)))
            .recurring(yearly)
            .unwrap(),
        );
    }
    // At most one short event at a time in each resource/day column, bringing
    // the maximum overlap to 15 lanes, below the admitted limit of 16.
    for i in 0..400usize {
        let day = start.checked_add_days(((i / 8) % 7) as i32).unwrap();
        let hour = (8 + i / 56) as u8;
        events.push(
            ScheduleEvent::new(
                key(format!("short-{i}")),
                "Short event",
                ScheduleInterval::new(dt(day, hour, 0), dt(day, hour, 30)).unwrap(),
            )
            .unwrap()
            .resource(key(format!("room-{}", i % 8))),
        );
    }
    let data = ScheduleData::new(
        ScheduleZone::Civil,
        ScheduleWindow::days(start, 7).unwrap(),
        events,
    )
    .unwrap();
    ResourceScheduleData::new(data, resources, ScheduleResourceWindow::new(0, 8).unwrap()).unwrap()
}
fn measure(name: &str, mut run: impl FnMut() -> usize) -> Duration {
    for _ in 0..3 {
        black_box(run());
    }
    let start = Instant::now();
    let mut count = 0usize;
    for _ in 0..RUNS {
        count += black_box(run());
    }
    let elapsed = start.elapsed();
    println!(
        "{name}: runs={RUNS}, total_ms={:.3}, mean_ms={:.3}, result_count={count}",
        elapsed.as_secs_f64() * 1000.0,
        elapsed.as_secs_f64() * 1000.0 / RUNS as f64
    );
    elapsed
}
fn main() {
    let model = fixture();
    let data = model.data();
    let time = ScheduleTimeWindow::default();
    let start = data.window().start().date();
    let occurrences = data.expand(MAX_SCHEDULE_OCCURRENCES).unwrap();
    let layout = model.layout_timeline(time, 7).unwrap();
    let max_lanes = layout
        .placements()
        .iter()
        .map(|p| p.position().lanes())
        .max()
        .unwrap_or(0);
    assert_eq!(data.events().len(), 512);
    assert_eq!(model.visible_resources().len(), 8);
    assert_eq!(layout.columns().len(), 56);
    assert_eq!(occurrences.len(), 512);
    assert!(max_lanes <= 16);
    assert!(layout.slot_count() <= 2048);
    println!(
        "fixture: events={}, visible_resources={}, days=7, occurrences={}, columns={}, slots={}, placements={}, max_lanes={max_lanes}",
        data.events().len(),
        model.visible_resources().len(),
        occurrences.len(),
        layout.columns().len(),
        layout.slot_count(),
        layout.placements().len()
    );
    println!(
        "profile=release; warmup=3 per operation; runs={RUNS}; timings are local observations, not a CI threshold"
    );
    let expand = measure("expand", || {
        black_box(data)
            .expand(black_box(MAX_SCHEDULE_OCCURRENCES))
            .unwrap()
            .len()
    });
    let day = measure("layout_day(room-0)", || {
        black_box(data)
            .layout_day(
                black_box(start),
                black_box(time),
                Some(black_box(&model.visible_resources()[0]).key()),
            )
            .unwrap()
            .len()
    });
    let timeline = measure("layout_timeline(8x7)", || {
        black_box(&model)
            .layout_timeline(black_box(time), black_box(7))
            .unwrap()
            .placements()
            .len()
    });
    println!(
        "ratios: timeline/expand={:.2}, timeline/day={:.2}",
        timeline.as_secs_f64() / expand.as_secs_f64(),
        timeline.as_secs_f64() / day.as_secs_f64()
    );
}
