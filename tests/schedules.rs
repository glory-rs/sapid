#![cfg(feature = "dates")]
use glory::prelude::*;
use glory::render::{DocumentPart, EventKind, NodeId, Props};
use sapid::prelude::*;
use std::{cell::RefCell, rc::Rc};
fn d(s: &str) -> CivilDate {
    CivilDate::parse_iso(s).unwrap()
}
fn dt(s: &str) -> LocalDateTime {
    parse_local_date_time(s).unwrap()
}
fn interval(a: &str, b: &str) -> ScheduleInterval {
    ScheduleInterval::new(dt(a), dt(b)).unwrap()
}
fn event(key: &str, title: &str, a: &str, b: &str) -> ScheduleEvent {
    ScheduleEvent::new(ScheduleKey::new(key).unwrap(), title, interval(a, b)).unwrap()
}
fn data() -> ScheduleData {
    ScheduleData::new(
        ScheduleZone::Civil,
        ScheduleWindow::days(d("2024-02-26"), 7).unwrap(),
        [
            event(
                "overnight",
                "Overnight",
                "2024-02-25T23:00",
                "2024-02-26T01:00",
            ),
            event(
                "planning",
                "Planning",
                "2024-02-29T09:00",
                "2024-02-29T10:00",
            ),
            ScheduleEvent::new(
                ScheduleKey::new("all-day").unwrap(),
                "Holiday",
                ScheduleInterval::all_day(d("2024-03-01"), d("2024-03-02")).unwrap(),
            )
            .unwrap()
            .all_day()
            .unwrap(),
        ],
    )
    .unwrap()
}
fn all(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut todo = tree.roots().to_vec();
    let mut out = vec![];
    while let Some(id) = todo.pop() {
        todo.extend(tree.node(id).unwrap().children.iter().copied());
        out.push(id)
    }
    out
}
fn button(app: &App, name: &str) -> NodeId {
    all(app).into_iter().find(|id|{let tree=app.tree();matches!(tree.node(*id).unwrap().descriptor.view().props,Props::Button(p)if p.label.starts_with(name))}).unwrap_or_else(||panic!("missing {name}"))
}
fn count_class(app: &App, class: &str) -> usize {
    all(app)
        .into_iter()
        .filter(|id| {
            app.tree()
                .node(*id)
                .unwrap()
                .descriptor
                .view()
                .classes
                .iter()
                .any(|v| v == class)
        })
        .count()
}
#[test]
fn agenda_uses_actual_lists_headings_bounded_sorted_window_and_all_day_text() {
    let app = App::memory();
    app.mount(agenda_view("agenda", data()).name("Work agenda"))
        .unwrap();
    assert_eq!(count_class(&app, "sapid-agenda-day"), 3);
    assert_eq!(count_class(&app, "sapid-agenda-event"), 3);
    let mut lists = 0;
    let mut items = 0;
    let mut headings = 0;
    for id in all(&app) {
        let tree = app.tree();
        let v = tree.node(id).unwrap().descriptor.view();
        match v.semantics.as_ref().and_then(|m| m.document_part) {
            Some(DocumentPart::UnorderedList) => lists += 1,
            Some(DocumentPart::ListItem) => items += 1,
            _ => {}
        }
        if matches!(v.props, Props::Text(p) if matches!(p.kind,glory::render::TextKind::Heading(_)))
        {
            headings += 1;
        }
    }
    assert_eq!((lists, items, headings), (3, 3, 3));
    assert!(button(&app, "Holiday") != button(&app, "Planning"));
    let empty = App::memory();
    empty
        .mount(schedule(
            "empty",
            ScheduleData::new(
                ScheduleZone::Civil,
                ScheduleWindow::days(d("2024-02-29"), 1).unwrap(),
                [],
            )
            .unwrap(),
        ))
        .unwrap();
    assert_eq!(count_class(&empty, "sapid-schedule-empty"), 1);
}
#[test]
fn typed_selection_precedes_generic_observation_and_controlled_refusal_is_read_only() {
    let app = App::memory();
    let selected = app.scope().cage(None);
    let calls = app.scope().cage(0);
    let seen = app.scope().cage(0);
    app.mount(
        schedule("selection", data())
            .selection(selected)
            .on_selection_change(move |v| {
                calls.revise(|n| *n += 1);
                selected.revise(|old| *old = Some(v));
            })
            .on(EventKind::Activate, move |_| {
                seen.revise(|n| *n = calls.get_untracked())
            }),
    )
    .unwrap();
    app.dispatch(button(&app, "Planning"), Event::Activate)
        .unwrap();
    app.flush().unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(seen.get(), 1);
    assert!(
        matches!(selected.get(),Some(ScheduleSelection::Event{key,..})if key.as_str()=="planning")
    );
    let other = App::memory();
    let selected = other.scope().cage(None);
    let proposals = other.scope().cage(0);
    other
        .mount(
            schedule("refuse", data())
                .selection(selected)
                .on_selection_change(move |_| proposals.revise(|n| *n += 1)),
        )
        .unwrap();
    other
        .dispatch(button(&other, "Planning"), Event::Activate)
        .unwrap();
    other.flush().unwrap();
    assert_eq!(selected.get(), None);
    assert_eq!(proposals.get(), 1);
}
#[test]
fn accepted_data_updates_keep_keyed_event_identity_and_callback_uses_latest_interval() {
    let app = App::memory();
    let source = app.scope().cage(data());
    let selected = app.scope().cage(None);
    app.mount(
        schedule("current", source)
            .on_selection_change(move |v| selected.revise(|old| *old = Some(v))),
    )
    .unwrap();
    let old = button(&app, "Planning");
    source.revise(|value| {
        *value = ScheduleData::new(
            ScheduleZone::Civil,
            value.window(),
            [event(
                "planning",
                "Changed planning",
                "2024-02-29T09:00",
                "2024-02-29T11:00",
            )],
        )
        .unwrap()
    });
    app.flush().unwrap();
    assert_eq!(button(&app, "Changed planning"), old);
    app.dispatch(old, Event::Activate).unwrap();
    app.flush().unwrap();
    assert!(
        matches!(selected.get(),Some(ScheduleSelection::Event{interval,..})if interval.end()==dt("2024-02-29T11:00"))
    );
    assert_eq!(count_class(&app, "sapid-agenda-event"), 1);
}
#[test]
fn same_tick_window_requests_are_ordered_without_implicitly_accepting_or_loading() {
    let app = App::memory();
    let source = app.scope().cage(data());
    let calls = Rc::new(RefCell::new(vec![]));
    let output = calls.clone();
    app.mount(schedule("requests", source).on_window_request(move |v| output.borrow_mut().push(v)))
        .unwrap();
    let next = button(&app, "Next schedule window");
    app.scope()
        .batch(|| {
            app.dispatch(next, Event::Activate).unwrap();
            app.dispatch(next, Event::Activate).unwrap();
        })
        .unwrap();
    let requests = calls.borrow();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].revision, 1);
    assert_eq!(requests[1].revision, 2);
    assert_eq!(requests[0].window.start().date(), d("2024-03-04"));
    assert_eq!(requests[1].window.start().date(), d("2024-03-11"));
    assert_eq!(source.with(|v| v.window().start().date()), d("2024-02-26"));
    drop(requests);
    app.dispatch(next, Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(calls.borrow()[2].window.start().date(), d("2024-03-04"));
    assert_eq!(calls.borrow()[2].revision, 3);
}
#[test]
fn overflow_source_rejects_before_children_then_recovers_without_callbacks() {
    let app = App::memory();
    let source = app.scope().cage(data());
    let calls = app.scope().cage(0);
    app.mount(schedule("overflow", source).on_selection_change(move |_| calls.revise(|n| *n += 1)))
        .unwrap();
    let old = button(&app, "Planning");
    let rule =
        ScheduleRecurrence::new(RecurrenceFrequency::Daily, 1, RecurrenceEnd::Never).unwrap();
    let events = (0..8).map(|i| {
        event(
            &format!("daily-{i}"),
            "Daily",
            "2024-01-01T09:00",
            "2024-01-01T10:00",
        )
        .recurring(rule)
        .unwrap()
    });
    let overflow = ScheduleData::new(
        ScheduleZone::Civil,
        ScheduleWindow::days(d("2024-01-01"), 366).unwrap(),
        events,
    )
    .unwrap();
    source.revise(|v| *v = overflow);
    assert!(app.flush().is_err());
    assert_eq!(button(&app, "Planning"), old);
    source.revise(|v| *v = data());
    app.flush().unwrap();
    assert_eq!(button(&app, "Planning"), old);
    assert_eq!(calls.get(), 0);
}
#[test]
fn readonly_disabled_and_event_permission_guard_real_buttons() {
    let app = App::memory();
    let readonly = app.scope().cage(true);
    let enabled = app.scope().cage(true);
    let calls = app.scope().cage(0);
    app.mount(
        schedule("locked", data())
            .readonly(readonly)
            .enabled(enabled)
            .on_selection_change(move |_| calls.revise(|n| *n += 1)),
    )
    .unwrap();
    assert!(
        !app.dispatch(button(&app, "Planning"), Event::Activate)
            .unwrap()
    );
    assert_eq!(calls.get(), 0);
    assert!(app.tree().enabled(button(&app, "Next schedule window")));
    readonly.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(
        app.dispatch(button(&app, "Planning"), Event::Activate)
            .unwrap()
    );
    app.flush().unwrap();
    assert_eq!(calls.get(), 1);
    enabled.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(!app.tree().enabled(button(&app, "Planning")));
    assert!(!app.tree().enabled(button(&app, "Next schedule window")));
}
#[test]
fn domain_edge_headers_are_disabled_and_owner_retirement_removes_live_items() {
    let app = App::memory();
    let visible = app.scope().cage(true);
    let root = app.scope().cage(None);
    let source = app.scope().cage(
        ScheduleData::new(
            ScheduleZone::Civil,
            ScheduleWindow::days(d("0001-01-01"), 1).unwrap(),
            [],
        )
        .unwrap(),
    );
    app.mount(glory::ui::when(visible, move || {
        schedule("owner", source).node_ref(root)
    }))
    .unwrap();
    assert!(!app.tree().enabled(button(&app, "Previous schedule window")));
    source.revise(|v| {
        *v = ScheduleData::new(
            ScheduleZone::Civil,
            ScheduleWindow::days(d("9999-12-30"), 1).unwrap(),
            [],
        )
        .unwrap()
    });
    app.flush().unwrap();
    assert!(!app.tree().enabled(button(&app, "Next schedule window")));
    let old = root.get().unwrap();
    visible.revise(|v| *v = false);
    app.flush().unwrap();
    assert_eq!(root.get(), None);
    assert!(!app.dispatch(old, Event::Activate).unwrap());
    assert_eq!(app.stats().queries, 0);
}
