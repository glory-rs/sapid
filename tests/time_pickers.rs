#![cfg(feature = "dates")]
use glory::prelude::*;
use glory::render::{InputProps, NodeId, Props};
use sapid::prelude::*;
use std::{cell::RefCell, rc::Rc};
fn t(s: &str) -> LocalTime {
    LocalTime::parse_iso(s).unwrap()
}
fn props(app: &App, id: NodeId) -> InputProps {
    let tree = app.tree();
    let Props::Input(p) = tree.node(id).unwrap().descriptor.view().props else {
        panic!()
    };
    p.clone()
}
fn edit(app: &App, id: NodeId, value: &str) {
    app.dispatch(
        id,
        Event::ValueChanged {
            value: value.into(),
            revision: props(app, id).revision,
            composing: false,
        },
    )
    .unwrap();
}
fn commit_edit(app: &App, id: NodeId, value: &str) {
    app.dispatch(
        id,
        Event::Commit {
            value: value.into(),
            revision: props(app, id).revision,
        },
    )
    .unwrap();
}
fn all(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut todo = tree.roots().to_vec();
    let mut result = vec![];
    while let Some(id) = todo.pop() {
        todo.extend(tree.node(id).unwrap().children.iter().copied());
        result.push(id)
    }
    result
}
fn button(app: &App, name: &str) -> NodeId {
    all(app)
        .into_iter()
        .find(|id| {
            let tree = app.tree();
            let d = tree.node(*id).unwrap().descriptor.view();
            matches!(d.props,Props::Button(p)if p.label==name)
                || d.name.as_deref() == Some(name)
        })
        .unwrap()
}
#[test]
fn real_segment_spinbuttons_keep_partial_empty_and_complete_time_values_distinct() {
    let app = App::memory();
    let h = app.scope().cage(None);
    let m = app.scope().cage(None);
    let v = app.scope().cage(None);
    let drafts = Rc::new(RefCell::new(Vec::new()));
    let calls = drafts.clone();
    app.mount(
        time_picker("segments")
            .hour_ref(h)
            .minute_ref(m)
            .value(v)
            .on_value_change(move |next| v.revise(|old| *old = next))
            .on_draft_change(move |draft| calls.borrow_mut().push(draft.status)),
    )
    .unwrap();
    let h = h.get().unwrap();
    let m = m.get().unwrap();
    let tree = app.tree();
    let hd = tree.node(h).unwrap().descriptor.view();
    assert_eq!(hd.effective_role(), Some(Role::SpinButton));
    assert_eq!(
        hd.semantics
            .as_ref()
            .unwrap()
            .spinbutton
            .as_ref()
            .unwrap()
            .max,
        Some(23.)
    );
    drop(tree);
    edit(&app, h, "12");
    app.flush().unwrap();
    assert_eq!(v.get(), None);
    assert_eq!(drafts.borrow().last(), Some(&TimePickerStatus::Incomplete));
    edit(&app, m, "34");
    app.flush().unwrap();
    assert_eq!(v.get(), Some(t("12:34")));
    assert_eq!(props(&app, h).value, "12");
    assert_eq!(props(&app, m).value, "34");
    edit(&app, h, "");
    app.flush().unwrap();
    assert_eq!(v.get(), Some(t("12:34")));
    edit(&app, m, "");
    app.flush().unwrap();
    assert_eq!(v.get(), None);
}
#[test]
fn noon_midnight_afternoon_toggle_and_second_segments_keep_full_wall_clock_precision() {
    let app = App::memory();
    let h = app.scope().cage(None);
    let m = app.scope().cage(None);
    let s = app.scope().cage(None);
    let v = app.scope().cage(Some(t("12:00:00")));
    app.mount(
        time_picker("twelve")
            .format(TimeFormat::new(
                HourCycle::H12,
                TimePrecision::Second,
                DateLocale::En,
            ))
            .hour_ref(h)
            .minute_ref(m)
            .second_ref(s)
            .value(v)
            .on_value_change(move |next| v.revise(|old| *old = next)),
    )
    .unwrap();
    let toggle = button(&app, "Afternoon (PM)");
    assert_eq!(props(&app, h.get().unwrap()).value, "12");
    app.dispatch(toggle, Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(v.get(), Some(t("00:00:00")));
    app.scope()
        .batch(|| {
            edit(&app, h.get().unwrap(), "01");
            edit(&app, m.get().unwrap(), "02");
            edit(&app, s.get().unwrap(), "03");
        })
        .unwrap();
    assert_eq!(v.get(), Some(t("01:02:03")));
    app.dispatch(toggle, Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(v.get(), Some(t("13:02:03")));
}
#[test]
fn same_tick_segments_compose_and_commit_pads_without_resetting_unedited_segments() {
    let app = App::memory();
    let h = app.scope().cage(None);
    let m = app.scope().cage(None);
    let s = app.scope().cage(None);
    let v = app.scope().cage(None);
    let commits = app.scope().cage(0usize);
    app.mount(
        time_picker("batch")
            .format(TimeFormat::new(
                HourCycle::H24,
                TimePrecision::Second,
                DateLocale::En,
            ))
            .hour_ref(h)
            .minute_ref(m)
            .second_ref(s)
            .value(v)
            .on_value_change(move |next| v.revise(|old| *old = next))
            .on_commit(move |_| commits.revise(|n| *n += 1)),
    )
    .unwrap();
    app.scope()
        .batch(|| {
            edit(&app, h.get().unwrap(), "23");
            edit(&app, m.get().unwrap(), "59");
            edit(&app, s.get().unwrap(), "8");
        })
        .unwrap();
    assert_eq!(v.get(), Some(t("23:59:08")));
    assert_eq!(props(&app, s.get().unwrap()).value, "8");
    commit_edit(&app, s.get().unwrap(), "8");
    app.flush().unwrap();
    assert_eq!(props(&app, s.get().unwrap()).value, "08");
    assert_eq!(props(&app, m.get().unwrap()).value, "59");
    assert_eq!(commits.get(), 1);
}
#[test]
fn controlled_refusal_invalid_source_recovery_and_overbudget_segment_use_last_accepted_state() {
    let app = App::memory();
    let h = app.scope().cage(None);
    let m = app.scope().cage(None);
    let v = app.scope().cage(Some(t("10:20")));
    let calls = app.scope().cage(0usize);
    let rejects = app.scope().cage(0usize);
    app.mount(
        time_picker("controlled")
            .hour_ref(h)
            .minute_ref(m)
            .value(v)
            .on_value_change(move |_| calls.revise(|n| *n += 1))
            .on_reject(move |_| rejects.revise(|n| *n += 1)),
    )
    .unwrap();
    let h = h.get().unwrap();
    edit(&app, h, "11");
    app.flush().unwrap();
    assert_eq!(props(&app, h).value, "10");
    assert_eq!(calls.get(), 1);
    v.revise(|v| *v = Some(t("10:20:01")));
    assert!(app.flush().is_err());
    assert_eq!(props(&app, h).value, "10");
    v.revise(|v| *v = Some(t("12:30")));
    app.flush().unwrap();
    assert_eq!(props(&app, h).value, "12");
    assert_eq!(props(&app, m.get().unwrap()).value, "30");
    assert_eq!(calls.get(), 1);
    edit(&app, h, "123");
    app.flush().unwrap();
    assert_eq!(props(&app, h).value, "12");
    assert_eq!(rejects.get(), 1);
}
#[test]
fn grid_selection_and_unavailable_typed_time_share_one_value_without_clamping() {
    let app = App::memory();
    let h = app.scope().cage(None);
    let m = app.scope().cage(None);
    let v = app.scope().cage(Some(t("09:00")));
    let status = app.scope().cage(TimePickerStatus::Empty);
    let rejects = app.scope().cage(0usize);
    let spec = TimeGridSpec::new(t("09:00"), t("10:00"), 1800, 2).unwrap();
    app.mount(
        time_picker("grid")
            .grid(spec)
            .hour_ref(h)
            .minute_ref(m)
            .value(v)
            .disable_time(t("09:30"))
            .on_value_change(move |next| v.revise(|old| *old = next))
            .on_draft_change(move |draft| status.revise(|s| *s = draft.status))
            .on_reject(move |_| rejects.revise(|n| *n += 1)),
    )
    .unwrap();
    assert!(
        !app.tree()
            .node(button(&app, "09:30"))
            .unwrap()
            .descriptor
            .view()
            .props
            .enabled()
    );
    edit(&app, m.get().unwrap(), "30");
    app.flush().unwrap();
    assert_eq!(v.get(), Some(t("09:00")));
    assert_eq!(status.get(), TimePickerStatus::Unavailable(t("09:30")));
    commit_edit(&app, m.get().unwrap(), "30");
    app.flush().unwrap();
    assert_eq!(rejects.get(), 1);
    app.dispatch(button(&app, "10:00"), Event::Activate)
        .unwrap();
    app.flush().unwrap();
    assert_eq!(v.get(), Some(t("10:00")));
    assert_eq!(props(&app, h.get().unwrap()).value, "10");
    assert_eq!(props(&app, m.get().unwrap()).value, "00");
    assert_eq!(status.get(), TimePickerStatus::Valid(t("10:00")));
}
#[test]
fn field_label_targets_real_hour_and_successful_names_are_explicit_independent_segments() {
    let app = App::memory();
    let root = app.scope().cage(None);
    let hour = app.scope().cage(None);
    let minute = app.scope().cage(None);
    app.mount(
        field("time-field")
            .label("Meeting segments")
            .required(true)
            .form_name("legacy-hour")
            .control(move |f| {
                time_picker("field")
                    .field(f)
                    .form_name("meeting")
                    .node_ref(root)
                    .hour_ref(hour)
                    .minute_ref(minute)
            }),
    )
    .unwrap();
    let tree = app.tree();
    let group_view = tree.node(root.get().unwrap()).unwrap().descriptor.view();
    assert_eq!(group_view.role, Some(Role::Group));
    assert!(group_view.semantics.as_ref().unwrap().labelled_by.is_some());
    let hour = tree.node(hour.get().unwrap()).unwrap().descriptor.view();
    let hs = hour.semantics.as_ref().unwrap();
    assert!(hs.required);
    assert_eq!(hs.form_control.as_ref().unwrap().name, "meeting_hour");
    assert!(hs.id.is_some());
    let minute = tree.node(minute.get().unwrap()).unwrap().descriptor.view();
    assert!(minute.semantics.as_ref().unwrap().required);
    assert_eq!(
        minute
            .semantics
            .as_ref()
            .unwrap()
            .form_control
            .as_ref()
            .unwrap()
            .name,
        "meeting_minute"
    );
}
#[test]
fn readonly_disabled_and_owner_retirement_control_real_spin_and_period_actions() {
    let app = App::memory();
    let h = app.scope().cage(None);
    let root = app.scope().cage(None);
    let readonly = app.scope().cage(false);
    let enabled = app.scope().cage(true);
    let visible = app.scope().cage(true);
    let v = app.scope().cage(Some(t("10:20")));
    app.mount(glory::ui::when(visible, move || {
        time_picker("owner")
            .hour_ref(h)
            .node_ref(root)
            .value(v)
            .readonly(readonly)
            .enabled(enabled)
            .on_value_change(move |next| v.revise(|old| *old = next))
    }))
    .unwrap();
    let old = h.get().unwrap();
    readonly.revise(|v| *v = true);
    app.flush().unwrap();
    assert!(
        !app.dispatch(
            old,
            Event::ValueChanged {
                value: "11".into(),
                revision: props(&app, old).revision,
                composing: false
            }
        )
        .unwrap()
    );
    assert_eq!(v.get(), Some(t("10:20")));
    enabled.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(!app.tree().enabled(old));
    visible.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(root.get().is_none() && h.get().is_none());
    assert_eq!(app.stats().queries, 0);
    assert!(!app.dispatch(old, Event::Activate).unwrap());
}

#[test]
fn explicit_segment_steps_use_real_spin_protocol_and_reject_zero_before_building() {
    let app = App::memory();
    let minute = app.scope().cage(None);
    let value = app.scope().cage(Some(t("10:20")));
    let calls = app.scope().cage(0usize);
    app.mount(
        time_picker("steps")
            .minute_step(15)
            .minute_ref(minute)
            .value(value)
            .on_value_change(move |v| {
                value.revise(|old| *old = v);
                calls.revise(|n| *n += 1)
            }),
    )
    .unwrap();
    let id = minute.get().unwrap();
    app.dispatch(
        id,
        Event::Spin {
            from: id,
            command: glory::render::RangeCommand::Increase,
        },
    )
    .unwrap();
    app.flush().unwrap();
    assert_eq!(value.get(), Some(t("10:35")));
    assert_eq!(calls.get(), 1);
    assert_eq!(
        app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .spinbutton
            .as_ref()
            .unwrap()
            .step,
        15.
    );
    let bad = App::memory();
    assert!(bad.mount(time_picker("bad-step").minute_step(0)).is_err());
    assert_eq!(bad.stats().nodes, 0);
}
