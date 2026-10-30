#![cfg(feature = "dates")]
use glory::prelude::*;
use glory::render::{InputProps, NodeId, Props};
use sapid::prelude::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
fn d(s: &str) -> CivilDate {
    CivilDate::parse_iso(s).unwrap()
}
fn dt(s: &str) -> LocalDateTime {
    parse_local_date_time(s).unwrap()
}
fn provider() -> DatesProvider {
    DatesProvider::new(d("2024-02-29"))
}
fn props(app: &App, id: NodeId) -> InputProps {
    let tree = app.tree();
    let Props::Input(p) = tree.node(id).unwrap().descriptor.view().props else {
        panic!()
    };
    p.clone()
}
fn edit(app: &App, id: NodeId, v: &str) {
    app.dispatch(
        id,
        Event::ValueChanged {
            value: v.into(),
            revision: props(app, id).revision,
            composing: false,
        },
    )
    .unwrap();
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
fn button(app: &App, label: &str) -> NodeId {
    all(app)
        .into_iter()
        .find(|id| {
            let tree = app.tree();
            let p = tree.node(*id).unwrap().descriptor.view();
            matches!(p.props,Props::Button(v)if v.label==label)
                || matches!(p.props, Props::Button(_)) && p.name.as_deref() == Some(label)
        })
        .unwrap_or_else(|| panic!("missing {label}"))
}
fn day(app: &App, date: CivilDate) -> NodeId {
    button(app, &DateLocale::En.date_label(date))
}
#[test]
fn complete_pair_is_required_and_draft_callback_precedes_value_and_resolution() {
    let app = App::memory();
    let h = app.scope().cage(None);
    let m = app.scope().cage(None);
    let value = app.scope().cage(None);
    let calls = Rc::new(RefCell::new(vec![]));
    let a = calls.clone();
    let b = calls.clone();
    let c = calls.clone();
    app.mount(
        inline_date_time_picker("pair", provider(), DateTimePolicy::Civil)
            .hour_ref(h)
            .minute_ref(m)
            .value(value)
            .on_draft_change(move |_| a.borrow_mut().push("draft"))
            .on_value_change(move |v| {
                b.borrow_mut().push("value");
                value.revise(|old| *old = v)
            })
            .on_resolved(move |v| {
                assert_eq!(v.resolution, DateTimeResolution::Civil);
                c.borrow_mut().push("resolved")
            }),
    )
    .unwrap();
    edit(&app, h.get().unwrap(), "12");
    app.flush().unwrap();
    assert_eq!(value.get(), None);
    app.dispatch(day(&app, d("2024-02-29")), Event::Activate)
        .unwrap();
    app.flush().unwrap();
    assert_eq!(value.get(), None);
    calls.borrow_mut().clear();
    edit(&app, m.get().unwrap(), "34");
    app.flush().unwrap();
    assert_eq!(value.get(), Some(dt("2024-02-29T12:34")));
    assert_eq!(&*calls.borrow(), &["draft", "value", "resolved"]);
    edit(&app, h.get().unwrap(), "");
    app.flush().unwrap();
    assert_eq!(value.get(), Some(dt("2024-02-29T12:34")));
}
#[test]
fn same_tick_segments_and_date_use_current_draft_without_fabricating_midnight() {
    let app = App::memory();
    let h = app.scope().cage(None);
    let m = app.scope().cage(None);
    let value = app.scope().cage(None);
    app.mount(
        inline_date_time_picker(
            "batch",
            provider(),
            DateTimePolicy::FixedOffset(FixedOffset::new(60).unwrap()),
        )
        .hour_ref(h)
        .minute_ref(m)
        .value(value)
        .on_value_change(move |v| value.revise(|x| *x = v)),
    )
    .unwrap();
    app.scope()
        .batch(|| {
            edit(&app, h.get().unwrap(), "23");
            edit(&app, m.get().unwrap(), "59");
            app.dispatch(day(&app, d("2024-02-29")), Event::Activate)
                .unwrap();
        })
        .unwrap();
    assert_eq!(value.get(), Some(dt("2024-02-29T23:59")));
}
#[test]
fn clear_and_equal_value_reset_revision_clear_partial_child_drafts() {
    let app = App::memory();
    let h = app.scope().cage(None);
    let m = app.scope().cage(None);
    let revision = app.scope().cage(0u64);
    let value = app.scope().cage(None);
    app.mount(
        inline_date_time_picker("reset", provider(), DateTimePolicy::Civil)
            .hour_ref(h)
            .minute_ref(m)
            .reset_revision(revision)
            .value(value)
            .on_value_change(move |v| value.revise(|x| *x = v)),
    )
    .unwrap();
    edit(&app, h.get().unwrap(), "12");
    app.flush().unwrap();
    assert_eq!(props(&app, h.get().unwrap()).value, "12");
    revision.revise(|r| *r += 1);
    app.flush().unwrap();
    assert_eq!(props(&app, h.get().unwrap()).value, "");
    edit(&app, h.get().unwrap(), "13");
    app.flush().unwrap();
    app.dispatch(button(&app, "Clear date and time"), Event::Activate)
        .unwrap();
    app.flush().unwrap();
    assert_eq!(props(&app, h.get().unwrap()).value, "");
    assert_eq!(props(&app, m.get().unwrap()).value, "");
    assert_eq!(value.get(), None);
}
#[test]
fn controlled_clear_refusal_restores_full_accepted_value_and_child_parts() {
    let app = App::memory();
    let h = app.scope().cage(None);
    let m = app.scope().cage(None);
    let value = app.scope().cage(Some(dt("2024-02-29T10:20")));
    let calls = app.scope().cage(0);
    app.mount(
        inline_date_time_picker("refuse", provider(), DateTimePolicy::Civil)
            .hour_ref(h)
            .minute_ref(m)
            .value(value)
            .on_value_change(move |_| calls.revise(|n| *n += 1)),
    )
    .unwrap();
    app.dispatch(button(&app, "Clear date and time"), Event::Activate)
        .unwrap();
    app.flush().unwrap();
    assert_eq!(value.get(), Some(dt("2024-02-29T10:20")));
    assert_eq!(calls.get(), 1);
    assert_eq!(props(&app, h.get().unwrap()).value, "10");
    assert_eq!(props(&app, m.get().unwrap()).value, "20");
}
#[test]
fn gap_and_fold_are_drafts_and_done_rejects_without_changing_accepted_civil_value() {
    for fold in [false, true] {
        let app = App::memory();
        let h = app.scope().cage(None);
        let value = app.scope().cage(Some(dt("2024-02-29T10:20")));
        let status = app.scope().cage(DateTimeStatus::Empty);
        let rejected = app.scope().cage(0);
        let policy = DateTimePolicy::application(AmbiguityPolicy::Reject, move |local| {
            if local.time().hour() == 11 {
                if fold {
                    LocalResolution::Ambiguous {
                        earlier: ResolvedDateTime::from_fixed(local, FixedOffset::new(60).unwrap()),
                        later: ResolvedDateTime::from_fixed(local, FixedOffset::new(0).unwrap()),
                    }
                } else {
                    LocalResolution::Gap
                }
            } else {
                LocalResolution::Unique(ResolvedDateTime::from_fixed(
                    local,
                    FixedOffset::new(0).unwrap(),
                ))
            }
        });
        app.mount(
            inline_date_time_picker("rules", provider(), policy)
                .hour_ref(h)
                .value(value)
                .on_value_change(move |v| value.revise(|old| *old = v))
                .on_draft_change(move |d| status.revise(|s| *s = d.status))
                .on_reject(move |_| rejected.revise(|n| *n += 1)),
        )
        .unwrap();
        edit(&app, h.get().unwrap(), "11");
        app.flush().unwrap();
        assert_eq!(value.get(), Some(dt("2024-02-29T10:20")));
        assert!(if fold {
            matches!(status.get(), DateTimeStatus::Ambiguous { .. })
        } else {
            status.get() == DateTimeStatus::Gap
        });
        app.dispatch(button(&app, "Done"), Event::Activate).unwrap();
        app.flush().unwrap();
        assert_eq!(rejected.get(), 1);
        assert_eq!(props(&app, h.get().unwrap()).value, "11");
    }
}
#[test]
fn edge_time_bounds_validate_pair_without_faulting_child_when_date_changes() {
    let app = App::memory();
    let h = app.scope().cage(None);
    let value = app.scope().cage(Some(dt("2024-02-28T09:00")));
    let status = app.scope().cage(DateTimeStatus::Empty);
    let bounds =
        DateTimeBounds::new(Some(dt("2024-02-28T08:00")), Some(dt("2024-02-29T08:00"))).unwrap();
    app.mount(
        inline_date_time_picker("edge", provider(), DateTimePolicy::Civil)
            .bounds(bounds)
            .hour_ref(h)
            .value(value)
            .on_value_change(move |v| value.revise(|s| *s = v))
            .on_draft_change(move |v| status.revise(|s| *s = v.status)),
    )
    .unwrap();
    app.dispatch(day(&app, d("2024-02-29")), Event::Activate)
        .unwrap();
    app.flush().unwrap();
    assert_eq!(
        status.get(),
        DateTimeStatus::OutOfBounds(dt("2024-02-29T09:00"))
    );
    assert_eq!(value.get(), Some(dt("2024-02-28T09:00")));
    edit(&app, h.get().unwrap(), "07");
    app.flush().unwrap();
    assert_eq!(value.get(), Some(dt("2024-02-29T07:00")));
}
#[test]
fn resolver_reentry_cannot_publish_obsolete_result_after_source_write() {
    let app = App::memory();
    let h = app.scope().cage(None);
    let value = app.scope().cage(Some(dt("2024-02-29T10:20")));
    let resolved = app.scope().cage(0usize);
    let once = Rc::new(Cell::new(false));
    let once2 = once.clone();
    let policy = DateTimePolicy::application(AmbiguityPolicy::Reject, move |local| {
        if local.time().hour() == 11 && !once2.replace(true) {
            value.revise(|v| *v = Some(dt("2024-02-29T15:30")));
        }
        LocalResolution::Unique(ResolvedDateTime::from_fixed(
            local,
            FixedOffset::new(0).unwrap(),
        ))
    });
    app.mount(
        inline_date_time_picker("reenter", provider(), policy)
            .hour_ref(h)
            .value(value)
            .on_value_change(move |v| value.revise(|s| *s = v))
            .on_resolved(move |_| resolved.revise(|n| *n += 1)),
    )
    .unwrap();
    edit(&app, h.get().unwrap(), "11");
    app.flush().unwrap();
    assert_eq!(value.get(), Some(dt("2024-02-29T15:30")));
    assert_eq!(props(&app, h.get().unwrap()).value, "15");
    assert_eq!(resolved.get(), 0);
    assert!(once.get());
}
#[test]
fn invalid_external_precision_is_rejected_then_recovers_silently_and_owner_retires_refs() {
    let app = App::memory();
    let visible = app.scope().cage(true);
    let h = app.scope().cage(None);
    let root = app.scope().cage(None);
    let value = app.scope().cage(Some(dt("2024-02-29T10:20")));
    let calls = app.scope().cage(0);
    app.mount(glory::ui::when(visible, move || {
        inline_date_time_picker("life", provider(), DateTimePolicy::Civil)
            .node_ref(root)
            .hour_ref(h)
            .value(value)
            .on_value_change(move |_| calls.revise(|n| *n += 1))
    }))
    .unwrap();
    value.revise(|v| *v = Some(dt("2024-02-29T10:20:01")));
    assert!(app.flush().is_err());
    assert_eq!(props(&app, h.get().unwrap()).value, "10");
    value.revise(|v| *v = Some(dt("2024-02-29T12:00")));
    app.flush().unwrap();
    assert_eq!(props(&app, h.get().unwrap()).value, "12");
    assert_eq!(calls.get(), 0);
    let old = h.get().unwrap();
    visible.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(root.get().is_none() && h.get().is_none());
    assert!(!app.dispatch(old, Event::Activate).unwrap());
    assert_eq!(app.stats().queries, 0);
}

#[test]
fn popup_iso_field_done_and_escape_keep_realtime_values_and_stable_input() {
    let app = App::memory();
    let input = app.scope().cage(None);
    let trigger = app.scope().cage(None);
    let panel = app.scope().cage(None);
    let hour = app.scope().cage(None);
    let value = app.scope().cage(Some(dt("2024-02-29T10:20")));
    let commits = app.scope().cage(0);
    app.mount(
        field("date-time-field")
            .label("Appointment")
            .required(true)
            .control(move |f| {
                date_time_picker("popup", provider(), DateTimePolicy::Civil)
                    .field(f)
                    .form_name("appointment")
                    .value(value)
                    .node_ref(input)
                    .trigger_ref(trigger)
                    .panel_ref(panel)
                    .hour_ref(hour)
                    .on_value_change(move |v| value.revise(|old| *old = v))
                    .on_commit(move |_| commits.revise(|n| *n += 1))
            }),
    )
    .unwrap();
    let id = input.get().unwrap();
    assert_eq!(props(&app, id).value, "2024-02-29T10:20:00");
    let tree = app.tree();
    let metadata = tree
        .node(id)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .as_ref()
        .unwrap();
    assert!(metadata.readonly && metadata.required);
    assert_eq!(metadata.form_control.as_ref().unwrap().name, "appointment");
    assert!(metadata.labelled_by.is_some());
    drop(tree);
    app.dispatch(trigger.get().unwrap(), Event::Activate)
        .unwrap();
    app.flush().unwrap();
    edit(&app, hour.get().unwrap(), "11");
    app.flush().unwrap();
    assert_eq!(props(&app, id).value, "2024-02-29T11:20:00");
    app.dispatch(
        panel.get().unwrap(),
        Event::Dismiss(glory::render::DismissReason::Escape),
    )
    .unwrap();
    app.flush().unwrap();
    assert_eq!(value.get(), Some(dt("2024-02-29T11:20")));
    assert_eq!(commits.get(), 0);
    app.dispatch(trigger.get().unwrap(), Event::Activate)
        .unwrap();
    app.flush().unwrap();
    app.dispatch(button(&app, "Done"), Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(commits.get(), 1);
    assert_eq!(input.get(), Some(id));
}

#[test]
fn draft_callback_reentry_wins_and_unmounted_popup_rebuilds_from_accepted_value() {
    let app = App::memory();
    let input = app.scope().cage(None);
    let trigger = app.scope().cage(None);
    let panel = app.scope().cage(None);
    let h = app.scope().cage(None);
    let value = app.scope().cage(Some(dt("2024-02-29T10:20")));
    app.mount(
        date_time_picker("unmount", provider(), DateTimePolicy::Civil)
            .value(value)
            .content_policy(ContentPolicy::Unmount)
            .node_ref(input)
            .trigger_ref(trigger)
            .panel_ref(panel)
            .hour_ref(h)
            .on_draft_change(move |d| {
                if matches!(d.status,DateTimeStatus::Valid(v)if v.local.time().hour()==11) {
                    value.revise(|v| *v = Some(dt("2024-02-29T15:30")));
                }
            })
            .on_value_change(move |v| value.revise(|old| *old = v)),
    )
    .unwrap();
    assert_eq!(h.get(), None);
    app.dispatch(trigger.get().unwrap(), Event::Activate)
        .unwrap();
    app.flush().unwrap();
    let old = h.get().unwrap();
    edit(&app, old, "11");
    app.flush().unwrap();
    assert_eq!(value.get(), Some(dt("2024-02-29T15:30")));
    assert_eq!(props(&app, old).value, "15");
    app.dispatch(
        panel.get().unwrap(),
        Event::Dismiss(glory::render::DismissReason::Escape),
    )
    .unwrap();
    app.flush().unwrap();
    assert_eq!(h.get(), None);
    app.dispatch(trigger.get().unwrap(), Event::Activate)
        .unwrap();
    app.flush().unwrap();
    assert_ne!(h.get(), Some(old));
    assert_eq!(props(&app, h.get().unwrap()).value, "15");
}

#[test]
fn resolver_can_retire_owner_without_late_value_or_callback() {
    let app = Rc::new(App::memory());
    let h = app.scope().cage(None);
    let root = app.scope().cage(None);
    let value = app.scope().cage(Some(dt("2024-02-29T10:20")));
    let calls = app.scope().cage(0);
    let weak = Rc::downgrade(&app);
    let policy = DateTimePolicy::application(AmbiguityPolicy::Reject, move |local| {
        if local.time().hour() == 11 {
            weak.upgrade().unwrap().clear().unwrap();
        }
        LocalResolution::Unique(ResolvedDateTime::from_fixed(
            local,
            FixedOffset::new(0).unwrap(),
        ))
    });
    app.mount(
        inline_date_time_picker("dispose", provider(), policy)
            .value(value)
            .hour_ref(h)
            .node_ref(root)
            .on_value_change(move |_| calls.revise(|n| *n += 1)),
    )
    .unwrap();
    let old = h.get().unwrap();
    edit(&app, old, "11");
    app.flush().unwrap();
    assert!(h.get().is_none() && root.get().is_none());
    assert_eq!(calls.get(), 0);
    assert_eq!(value.get(), Some(dt("2024-02-29T10:20")));
    assert_eq!(app.stats().queries, 0);
}

#[test]
fn resolver_and_draft_reentry_that_only_changes_reset_configuration_rejects_old_proposal() {
    for in_draft in [false, true] {
        let app = App::memory();
        let h = app.scope().cage(None);
        let revision = app.scope().cage(0u64);
        let value = app.scope().cage(Some(dt("2024-02-29T10:20")));
        let calls = app.scope().cage(0);
        let once = Rc::new(Cell::new(false));
        let once_resolver = once.clone();
        let once_draft = once.clone();
        let policy = DateTimePolicy::application(AmbiguityPolicy::Reject, move |local| {
            if !in_draft && local.time().hour() == 11 && !once_resolver.replace(true) {
                revision.revise(|n| *n += 1);
            }
            LocalResolution::Unique(ResolvedDateTime::from_fixed(local, FixedOffset::UTC))
        });
        app.mount(
            inline_date_time_picker("config-reentry", provider(), policy)
                .reset_revision(revision)
                .value(value)
                .hour_ref(h)
                .on_draft_change(move |d| {
                    if in_draft
                        && matches!(d.status,DateTimeStatus::Valid(v)if v.local.time().hour()==11)
                        && !once_draft.replace(true)
                    {
                        revision.revise(|n| *n += 1);
                    }
                })
                .on_value_change(move |v| {
                    calls.revise(|n| *n += 1);
                    value.revise(|old| *old = v)
                }),
        )
        .unwrap();
        edit(&app, h.get().unwrap(), "11");
        app.flush().unwrap();
        assert!(once.get());
        assert_eq!(value.get(), Some(dt("2024-02-29T10:20")));
        assert_eq!(props(&app, h.get().unwrap()).value, "10");
        assert_eq!(calls.get(), 0);
    }
}
