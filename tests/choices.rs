use glory::render::{CheckMode, CheckProps, Event, EventKind, NodeId, Props};
use glory::ui::{App, Widget};
use sapid::{CheckState, checkbox_group, checkbox_item, field, mixed_checkbox, switch};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
fn find(app: &App, class: &str) -> Vec<NodeId> {
    let tree = app.tree();
    let mut found = Vec::new();
    let mut pending = tree.roots().iter().rev().copied().collect::<Vec<_>>();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if node.descriptor.view().classes.iter().any(|c| c == class) {
            found.push(id);
        }
        pending.extend(node.children.iter().rev().copied());
    }
    found
}
fn props(app: &App, id: NodeId) -> CheckProps {
    let Props::Check(props) = app.tree().node(id).unwrap().descriptor.view().props.clone()
    else {
        panic!("not a check")
    };
    props
}
#[test]
fn mixed_controlled_proposals_do_not_write_source_and_reconcile_full_state_after_tick() {
    let app = App::memory();
    let source = app.scope().cage(CheckState::Mixed);
    let order = Rc::new(RefCell::new(Vec::new()));
    let typed = order.clone();
    let observed = order.clone();
    app.mount(
        mixed_checkbox()
            .state(source)
            .default_state(CheckState::Checked)
            .on_state_change(move |state| typed.borrow_mut().push(format!("{state:?}")))
            .on(EventKind::CheckedChanged, move |_| {
                observed.borrow_mut().push("observer".into())
            }),
    )
    .unwrap();
    let id = find(&app, "sapid-mixed-checkbox")[0];
    for checked in [true, false] {
        app.dispatch(id, Event::CheckedChanged(checked)).unwrap();
        assert_eq!(source.get(), CheckState::Mixed);
        assert_eq!(props(&app, id).mode, CheckMode::Mixed);
        assert!(!props(&app, id).checked);
    }
    assert_eq!(
        *order.borrow(),
        ["Checked", "observer", "Unchecked", "observer"]
    );
    source.revise(|state| *state = CheckState::Checked);
    app.flush().unwrap();
    assert_eq!(props(&app, id).mode, CheckMode::Checkbox);
    assert!(props(&app, id).checked);
    assert_eq!(find(&app, "sapid-mixed-checkbox"), [id]);
}
#[test]
fn local_mixed_activates_to_binary_selection_and_switch_has_independent_semantics() {
    let app = App::memory();
    app.mount(glory::ui::group().fill((
        mixed_checkbox().default_state(CheckState::Mixed),
        switch().default_checked(false),
    )))
    .unwrap();
    let mixed = find(&app, "sapid-mixed-checkbox")[0];
    let toggle = find(&app, "sapid-switch")[0];
    app.dispatch(mixed, Event::CheckedChanged(true)).unwrap();
    assert_eq!(props(&app, mixed).mode, CheckMode::Checkbox);
    assert!(props(&app, mixed).checked);
    app.dispatch(toggle, Event::CheckedChanged(true)).unwrap();
    assert_eq!(props(&app, toggle).mode, CheckMode::Switch);
    assert!(props(&app, toggle).checked);
}
#[test]
fn switch_field_readonly_disabled_and_controlled_rejection_follow_shared_check_rules() {
    let app = App::memory();
    let source = app.scope().cage(false);
    let readonly = app.scope().cage(false);
    let calls = Rc::new(Cell::new(0));
    let seen = calls.clone();
    app.mount(
        field("notifications")
            .label("Notifications")
            .description("Status messages")
            .required(true)
            .readonly(readonly)
            .invalid(true)
            .control(move |field| {
                let seen = seen.clone();
                switch()
                    .field(field)
                    .checked(source)
                    .on_checked_change(move |_| seen.set(seen.get() + 1))
            }),
    )
    .unwrap();
    let id = find(&app, "sapid-switch")[0];
    let descriptor = app.tree().node(id).unwrap().descriptor.view().to_owned();
    let semantics = descriptor.semantics.unwrap();
    assert!(semantics.required && semantics.invalid);
    assert!(semantics.labelled_by.is_some() && semantics.described_by.is_some());
    app.dispatch(id, Event::CheckedChanged(true)).unwrap();
    assert!(!source.get() && !props(&app, id).checked);
    assert_eq!(calls.get(), 1);
    readonly.revise(|value| *value = true);
    app.flush().unwrap();
    assert!(!app.dispatch(id, Event::CheckedChanged(true)).unwrap());
    assert_eq!(calls.get(), 1);
}
#[test]
fn grouped_selection_aggregates_enabled_items_and_preserves_disabled_selection() {
    let app = App::memory();
    let selected = app.scope().cage(vec!["a".to_owned(), "fixed".to_owned()]);
    app.mount(
        checkbox_group("team")
            .label("Team permissions")
            .select_all_label("All editable")
            .selected(selected)
            .items([
                checkbox_item("a", "Read"),
                checkbox_item("b", "Write"),
                checkbox_item("fixed", "Inherited").enabled(false),
            ])
            .on_selected_change(move |value| selected.revise(|old| *old = value)),
    )
    .unwrap();
    let all = find(&app, "sapid-checkbox-group-all")[0];
    let items = find(&app, "sapid-checkbox-group-item");
    assert_eq!(props(&app, all).mode, CheckMode::Mixed);
    app.dispatch(all, Event::CheckedChanged(true)).unwrap();
    assert_eq!(selected.get(), ["a", "b", "fixed"]);
    assert!(props(&app, all).checked);
    assert_eq!(props(&app, all).mode, CheckMode::Checkbox);
    app.dispatch(all, Event::CheckedChanged(false)).unwrap();
    assert_eq!(selected.get(), ["fixed"]);
    assert!(!props(&app, all).checked);
    assert!(
        !app.dispatch(items[2], Event::CheckedChanged(false))
            .unwrap()
    );
    app.dispatch(items[1], Event::CheckedChanged(true)).unwrap();
    assert_eq!(selected.get(), ["b", "fixed"]);
    assert_eq!(props(&app, all).mode, CheckMode::Mixed);
}
#[test]
fn group_rejected_proposals_leave_child_and_select_all_models_unchanged() {
    let app = App::memory();
    let selected = app.scope().cage(vec!["a".to_owned()]);
    let proposals = Rc::new(RefCell::new(Vec::new()));
    let seen = proposals.clone();
    app.mount(
        checkbox_group("controlled")
            .select_all_label("All")
            .selected(selected)
            .items([checkbox_item("a", "A"), checkbox_item("b", "B")])
            .on_selected_change(move |value| seen.borrow_mut().push(value)),
    )
    .unwrap();
    let all = find(&app, "sapid-checkbox-group-all")[0];
    let items = find(&app, "sapid-checkbox-group-item");
    app.dispatch(all, Event::CheckedChanged(true)).unwrap();
    app.dispatch(items[0], Event::CheckedChanged(false))
        .unwrap();
    assert_eq!(selected.get(), ["a"]);
    assert_eq!(props(&app, all).mode, CheckMode::Mixed);
    assert!(props(&app, items[0]).checked);
    assert_eq!(
        *proposals.borrow(),
        [vec!["a".to_owned(), "b".to_owned()], Vec::<String>::new()]
    );
}
#[test]
fn explicit_group_rejects_duplicate_unknown_and_excessive_items_before_creating_nodes() {
    let app = App::memory();
    for widget in [
        checkbox_group("duplicate")
            .items([checkbox_item("a", "A"), checkbox_item("a", "Again")]),
        checkbox_group("unknown")
            .item(checkbox_item("a", "A"))
            .default_selected(["missing".to_owned()]),
        checkbox_group("selected-twice")
            .item(checkbox_item("a", "A"))
            .default_selected(["a".to_owned(), "a".to_owned()]),
        checkbox_group("budget").items(
            (0..=sapid::MAX_CHECKBOX_GROUP_ITEMS)
                .map(|index| checkbox_item(format!("item{index}"), "Item")),
        ),
    ] {
        assert!(app.mount(widget).is_err());
        assert_eq!(app.stats().nodes, 0);
    }
}
#[test]
fn choice_owners_release_sources_callbacks_node_references_and_skip_retired_observers() {
    let app = App::memory();
    let baseline = app.runtime_stats();
    let reference = app.scope().cage(None);
    let calls = Rc::new(Cell::new(0));
    for _ in 0..20 {
        let owner = app.scope().child().unwrap();
        let retire = owner.clone();
        let seen = calls.clone();
        mixed_checkbox()
            .node_ref(reference)
            .default_state(CheckState::Mixed)
            .on_state_change(move |_| {
                seen.set(seen.get() + 1);
                retire.dispose();
            })
            .on(EventKind::CheckedChanged, |_| {
                panic!("observer after retirement")
            })
            .show_in(&owner)
            .unwrap();
        app.flush().unwrap();
        let id = reference.get().unwrap();
        app.dispatch(id, Event::CheckedChanged(true)).unwrap();
        assert_eq!(reference.get(), None);
        assert_eq!(app.stats().nodes, 0);
        assert_eq!(app.stats().handlers, 0);
        assert_eq!(app.runtime_stats().nodes, baseline.nodes + 1);
        assert_eq!(Rc::strong_count(&calls), 1);
    }
    assert_eq!(calls.get(), 20);
}
#[test]
fn ssr_switch_and_mixed_use_true_check_controls_and_stable_fieldset_legend() {
    let snapshot = glory::ssr::snapshot_with(|_| {
        glory::ui::group().fill((
            mixed_checkbox()
                .default_state(CheckState::Mixed)
                .label("Partial"),
            field("switch-field")
                .label("Notifications")
                .required(true)
                .control(|field| switch().field(field).default_checked(true)),
            checkbox_group("group")
                .label("Capabilities")
                .select_all_label("All")
                .default_selected(["a".to_owned()])
                .items([checkbox_item("a", "Read"), checkbox_item("b", "Write")]),
        ))
    })
    .unwrap();
    assert!(snapshot.html.contains("aria-checked=\"mixed\""));
    assert!(snapshot.html.contains("data-gly-indeterminate=\"\""));
    assert!(snapshot.html.contains("role=\"switch\""));
    assert!(snapshot.html.contains("<fieldset"));
    assert!(snapshot.html.contains("<legend"));
    assert!(snapshot.html.contains("type=\"checkbox\""));
}
