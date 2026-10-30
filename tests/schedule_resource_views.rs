#![cfg(feature = "dates")]
use sapid::prelude::*;
fn d(s: &str) -> CivilDate {
    CivilDate::parse_iso(s).unwrap()
}
#[test]
fn resource_month_day_projection_separates_same_day_events_and_preserves_full_occurrences() {
    let events = [
        ("a", "09:00", "10:00"),
        ("b", "10:00", "11:00"),
        ("c", "11:00", "12:00"),
    ]
    .map(|(key, start, end)| {
        ScheduleEvent::new(
            ScheduleKey::new(key).unwrap(),
            key,
            ScheduleInterval::new(
                dt(&format!("2024-02-29T{start}")),
                dt(&format!("2024-02-29T{end}")),
            )
            .unwrap(),
        )
        .unwrap()
        .resource(ScheduleKey::new("room-0").unwrap())
    });
    let model = ResourceScheduleData::new(
        ScheduleData::new(
            ScheduleZone::Civil,
            ScheduleWindow::month(YearMonth::new(2024, 2).unwrap()).unwrap(),
            events,
        )
        .unwrap(),
        registrations(1),
        ScheduleResourceWindow::default(),
    )
    .unwrap();
    let layout = model.layout_month().unwrap();
    assert_eq!(layout.slot_count(), 29);
    assert_eq!(layout.placements().len(), 3);
    for p in layout.placements() {
        assert_eq!(p.position().lanes(), 3);
        assert!((p.position().start_fraction() - 28. / 29.).abs() < 1e-12);
        assert_eq!(p.position().end_fraction(), 1.);
    }
    assert_eq!(
        model.data().events()[0].interval().start(),
        dt("2024-02-29T09:00")
    );
    assert!(data(Some("room-0")).window().start().date() == d("2024-02-29"));
    assert!(
        ResourceScheduleData::new(
            data(Some("room-0")),
            registrations(1),
            ScheduleResourceWindow::default()
        )
        .unwrap()
        .layout_month()
        .is_err()
    );
}
#[test]
fn resource_month_rejects_seventeenth_projected_lane_and_all_day_edits_keep_midnights() {
    let events = (0..17).map(|i| {
        ScheduleEvent::new(
            ScheduleKey::new(format!("e{i}")).unwrap(),
            "Busy",
            ScheduleInterval::new(dt("2024-02-29T09:00"), dt("2024-02-29T10:00")).unwrap(),
        )
        .unwrap()
        .resource(ScheduleKey::new("room-0").unwrap())
    });
    let model = ResourceScheduleData::new(
        ScheduleData::new(
            ScheduleZone::Civil,
            ScheduleWindow::month(YearMonth::new(2024, 2).unwrap()).unwrap(),
            events,
        )
        .unwrap(),
        registrations(1),
        ScheduleResourceWindow::default(),
    )
    .unwrap();
    assert!(model.layout_month().is_err());
    let interval = ScheduleInterval::all_day(d("2024-02-29"), d("2024-03-01")).unwrap();
    let event = ScheduleEvent::new(ScheduleKey::new("all-day").unwrap(), "Holiday", interval)
        .unwrap()
        .all_day()
        .unwrap()
        .resource(ScheduleKey::new("room-0").unwrap())
        .permissions(SchedulePermissions {
            select: true,
            move_event: true,
            resize_event: true,
        });
    let data = ScheduleData::new(
        ScheduleZone::Civil,
        ScheduleWindow::month(YearMonth::new(2024, 2).unwrap()).unwrap(),
        [event],
    )
    .unwrap();
    let mut edit = ScheduleEdit {
        event: ScheduleKey::new("all-day").unwrap(),
        candidate: 0,
        kind: ScheduleEditKind::Move,
        original: interval,
        proposed: interval.shifted_minutes(30).unwrap(),
        original_resource: Some(ScheduleKey::new("room-0").unwrap()),
        proposed_resource: Some(ScheduleKey::new("room-0").unwrap()),
    };
    assert!(data.validate_edit(&registrations(1), &edit).is_err());
    edit.proposed = interval.shifted_minutes(1440).unwrap();
    data.validate_edit(&registrations(1), &edit).unwrap();
}
fn dt(s: &str) -> LocalDateTime {
    parse_local_date_time(s).unwrap()
}
fn registrations(n: usize) -> ScheduleResources {
    ScheduleResources::new((0..n).map(|i| {
        ScheduleResource::new(
            ScheduleKey::new(format!("room-{i}")).unwrap(),
            format!("Room {i}"),
        )
        .unwrap()
    }))
    .unwrap()
}
fn data(key: Option<&str>) -> ScheduleData {
    let mut event = ScheduleEvent::new(
        ScheduleKey::new("a").unwrap(),
        "Planning",
        ScheduleInterval::new(dt("2024-02-29T09:00"), dt("2024-02-29T10:00")).unwrap(),
    )
    .unwrap();
    if let Some(key) = key {
        event = event.resource(ScheduleKey::new(key).unwrap())
    }
    ScheduleData::new(
        ScheduleZone::Civil,
        ScheduleWindow::days(d("2024-02-29"), 1).unwrap(),
        [event],
    )
    .unwrap()
}
#[test]
fn resource_configuration_is_jointly_validated_and_never_hides_unassigned_or_unknown_events() {
    assert!(
        ResourceScheduleData::new(
            data(None),
            registrations(2),
            ScheduleResourceWindow::default()
        )
        .is_err()
    );
    assert!(
        ResourceScheduleData::new(
            data(Some("missing")),
            registrations(2),
            ScheduleResourceWindow::default()
        )
        .is_err()
    );
    let v = ResourceScheduleData::new(
        data(Some("room-1")),
        registrations(2),
        ScheduleResourceWindow::default(),
    )
    .unwrap();
    assert_eq!(v.visible_resources().len(), 1);
    assert_eq!(v.visible_resources()[0].key().as_str(), "room-0");
    assert_eq!(
        v.data().events().len(),
        1,
        "off-window registered event remains in current authoritative data"
    );
}
#[test]
fn explicit_resource_window_budgets_and_empty_registration_are_total_without_overflow() {
    for (first, count) in [(usize::MAX, 1), (0, 0), (0, 9), (63, 2), (64, 1)] {
        assert!(ScheduleResourceWindow::new(first, count).is_err());
    }
    let resources = registrations(64);
    let w = ScheduleResourceWindow::new(56, 8).unwrap();
    assert_eq!(w.resolve(&resources).unwrap().len(), 8);
    assert_eq!(w.resolve(&resources).unwrap()[7].key().as_str(), "room-63");
    assert!(
        ScheduleResourceWindow::new(1, 2)
            .unwrap()
            .resolve(&registrations(2))
            .is_err()
    );
    let empty = ScheduleData::new(
        ScheduleZone::Civil,
        ScheduleWindow::days(d("2024-02-29"), 1).unwrap(),
        [],
    )
    .unwrap();
    assert!(
        ResourceScheduleData::new(
            empty,
            ScheduleResources::default(),
            ScheduleResourceWindow::default()
        )
        .unwrap()
        .visible_resources()
        .is_empty()
    );
    assert!(
        ScheduleResourceWindow::new(1, 1)
            .unwrap()
            .resolve(&ScheduleResources::default())
            .is_err()
    );
}
#[test]
fn resource_timeline_separates_equal_time_intervals_and_uses_only_visible_numeric_columns() {
    let resources = registrations(4);
    let window = ScheduleWindow::days(d("2024-02-29"), 1).unwrap();
    let source = ScheduleData::new(
        ScheduleZone::Civil,
        window,
        (0..4).map(|i| {
            ScheduleEvent::new(
                ScheduleKey::new(format!("e-{i}")).unwrap(),
                "Busy",
                ScheduleInterval::new(dt("2024-02-29T09:00"), dt("2024-02-29T10:00")).unwrap(),
            )
            .unwrap()
            .resource(ScheduleKey::new(format!("room-{i}")).unwrap())
        }),
    )
    .unwrap();
    let source = ResourceScheduleData::new(
        source,
        resources,
        ScheduleResourceWindow::new(1, 2).unwrap(),
    )
    .unwrap();
    let layout = source
        .layout_timeline(ScheduleTimeWindow::default(), 1)
        .unwrap();
    assert_eq!(layout.columns().len(), 2);
    assert_eq!(layout.columns()[0].resource_index(), 1);
    assert_eq!(layout.columns()[1].resource_index(), 2);
    assert_eq!(layout.slot_count(), 40);
    assert_eq!(layout.placements().len(), 2);
    assert!(
        layout
            .placements()
            .iter()
            .all(|p| p.position().lanes() == 1)
    );
    assert_eq!(layout.placements()[1].column(), 1);
}
#[test]
fn resource_week_slot_budget_is_product_of_live_resources_dates_and_time_rows() {
    let empty = ScheduleData::new(
        ScheduleZone::Civil,
        ScheduleWindow::days(d("2024-02-26"), 7).unwrap(),
        [],
    )
    .unwrap();
    let a = ResourceScheduleData::new(
        empty.clone(),
        registrations(8),
        ScheduleResourceWindow::new(0, 8).unwrap(),
    )
    .unwrap();
    let l = a.layout_timeline(ScheduleTimeWindow::default(), 7).unwrap();
    assert_eq!(l.columns().len(), 56);
    assert_eq!(l.slot_count(), 1120);
    assert_eq!(l.columns()[7].resource_index(), 1);
    assert_eq!(l.columns()[7].day_index(), 0);
    assert_eq!(l.columns()[6].date(), d("2024-03-03"));
    let full = ScheduleTimeWindow::new(0, 1440, 15).unwrap();
    assert!(a.layout_timeline(full, 7).is_err());
    let a = ResourceScheduleData::new(
        empty,
        registrations(3),
        ScheduleResourceWindow::new(0, 3).unwrap(),
    )
    .unwrap();
    assert_eq!(a.layout_timeline(full, 7).unwrap().slot_count(), 2016);
    assert!(a.layout_timeline(full, 1).is_err());
}
#[test]
fn overfull_resource_overlap_rejects_complete_layout_before_any_partial_geometry_is_returned() {
    let events = (0..17).map(|i| {
        ScheduleEvent::new(
            ScheduleKey::new(format!("e-{i}")).unwrap(),
            "Busy",
            ScheduleInterval::new(dt("2024-02-29T09:00"), dt("2024-02-29T10:00")).unwrap(),
        )
        .unwrap()
        .resource(ScheduleKey::new("room-0").unwrap())
    });
    let source = ScheduleData::new(
        ScheduleZone::Civil,
        ScheduleWindow::days(d("2024-02-29"), 1).unwrap(),
        events,
    )
    .unwrap();
    let a = ResourceScheduleData::new(source, registrations(1), ScheduleResourceWindow::default())
        .unwrap();
    assert!(a.layout_timeline(ScheduleTimeWindow::default(), 1).is_err());
}
#[test]
fn civil_edit_arithmetic_preserves_nanoseconds_and_rejects_crossed_or_exhausted_endpoints() {
    let span = ScheduleInterval::new(
        dt("2024-02-29T23:45:10.123456789"),
        dt("2024-03-01T00:15:10.123456789"),
    )
    .unwrap();
    let moved = span.shifted_minutes(30).unwrap();
    assert_eq!(moved.start(), dt("2024-03-01T00:15:10.123456789"));
    assert_eq!(moved.duration_nanoseconds(), span.duration_nanoseconds());
    assert_eq!(span.resize_start_by_minutes(-15).unwrap().end(), span.end());
    assert_eq!(
        span.resize_end_by_minutes(15).unwrap().start(),
        span.start()
    );
    assert!(span.resize_start_by_minutes(30).is_err());
    assert!(span.resize_end_by_minutes(-30).is_err());
    assert!(span.shifted_minutes(i64::MAX).is_err());
    assert!(span.shifted_minutes(i64::MIN).is_err());
    let edge =
        ScheduleInterval::new(dt("9999-12-31T23:00"), dt("9999-12-31T23:59:59.999999999")).unwrap();
    assert!(edge.shifted_minutes(1).is_err());
    assert!(edge.resize_end_by_minutes(1).is_err());
    assert!(
        edge.resize_start_by_minutes(1).is_ok(),
        "resizing the start must not translate the fixed exclusive end"
    );
}
