use glory::render::{Event, EventKind, NodeId, Props};
use glory::ui::App;
use sapid::{ChipMode, chip, chip_group, chip_item, field};
fn find(app: &App, class: &str) -> Vec<NodeId> {
    let tree = app.tree();
    let mut pending = tree.roots().iter().rev().copied().collect::<Vec<_>>();
    let mut result = Vec::new();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if node.descriptor.view().classes.iter().any(|c| c == class) {
            result.push(id);
        }
        pending.extend(node.children.iter().rev().copied());
    }
    result
}
fn checked(app: &App, id: NodeId) -> bool {
    matches!(&app.tree().node(id).unwrap().descriptor.view().props.clone(),Props::Check(check) if check.checked)
}
#[test]
fn chip_controlled_checkbox_proposals_preserve_state_classes_and_field_metadata() {
    let app = App::memory();
    let source = app.scope().cage(false);
    let events = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let typed = events.clone();
    let observer = events.clone();
    app.mount(
        field("standalone-chip")
            .label("Featured")
            .required(true)
            .control(move |field| {
                let typed = typed.clone();
                let observer = observer.clone();
                chip()
                    .field(field)
                    .class("custom-chip")
                    .checked(source)
                    .on_checked_change(move |value| typed.borrow_mut().push(value))
                    .on(EventKind::CheckedChanged, move |_| {
                        observer.borrow_mut().push(false)
                    })
            }),
    )
    .unwrap();
    let id = find(&app, "sapid-chip")[0];
    app.dispatch(id, Event::CheckedChanged(true)).unwrap();
    assert!(!checked(&app, id) && !source.get());
    assert_eq!(*events.borrow(), [true, false]);
    assert!(
        app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .classes
            .iter()
            .any(|class| class == "custom-chip")
    );
    assert!(
        app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .required
    );
    source.revise(|value| *value = true);
    app.flush().unwrap();
    assert!(checked(&app, id));
    assert_eq!(find(&app, "sapid-chip"), [id]);
}
#[test]
fn single_chip_group_is_a_radio_group_and_multiple_is_independent_checkboxes() {
    let app = App::memory();
    app.mount(
        chip_group("single")
            .default_value("a")
            .items([chip_item("a", "A"), chip_item("b", "B")]),
    )
    .unwrap();
    let members = find(&app, "sapid-radio");
    assert_eq!(members.len(), 2);
    app.dispatch(members[1], Event::CheckedChanged(true))
        .unwrap();
    assert!(!checked(&app, members[0]) && checked(&app, members[1]));
    app.clear().unwrap();
    app.mount(
        chip_group("multiple")
            .mode(ChipMode::Multiple)
            .default_values(["a".to_owned()])
            .items([chip_item("a", "A"), chip_item("b", "B")]),
    )
    .unwrap();
    let members = find(&app, "sapid-checkbox-group-item");
    app.dispatch(members[1], Event::CheckedChanged(true))
        .unwrap();
    assert!(checked(&app, members[0]) && checked(&app, members[1]));
    app.dispatch(members[0], Event::CheckedChanged(false))
        .unwrap();
    assert!(!checked(&app, members[0]) && checked(&app, members[1]));
}
#[test]
fn multiple_chip_field_requires_any_enabled_selection_and_preserves_readonly_disabled() {
    let app = App::memory();
    let readonly = app.scope().cage(false);
    let disabled = app.scope().cage(false);
    app.mount(
        field("interests")
            .label("Interests")
            .required(true)
            .readonly(readonly)
            .invalid(true)
            .control(move |field| {
                chip_group("interests-list")
                    .mode(ChipMode::Multiple)
                    .field(field)
                    .items([
                        chip_item("blocked", "Blocked").enabled(false),
                        chip_item("a", "A"),
                        chip_item("b", "B").enabled(app_enabled(disabled)),
                    ])
            }),
    )
    .unwrap();
    let members = find(&app, "sapid-checkbox-group-item");
    let required = |index: usize| {
        app.tree()
            .node(members[index])
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .is_some_and(|states| states.required)
    };
    assert!(!required(0) && required(1) && !required(2));
    app.dispatch(members[2], Event::CheckedChanged(true))
        .unwrap();
    assert!(!required(0) && !required(1) && !required(2));
    readonly.revise(|value| *value = true);
    app.flush().unwrap();
    assert!(
        !app.dispatch(members[1], Event::CheckedChanged(true))
            .unwrap()
    );
    assert!(!checked(&app, members[1]));
    readonly.revise(|value| *value = false);
    app.flush().unwrap();
    app.dispatch(members[2], Event::CheckedChanged(false))
        .unwrap();
    assert!(required(1));
    disabled.revise(|value| *value = true);
    app.flush().unwrap();
    assert!(
        !app.dispatch(members[2], Event::CheckedChanged(true))
            .unwrap()
    );
}
fn app_enabled(disabled: glory::Cage<bool>) -> glory::Lotus<bool> {
    disabled.map(|disabled| !disabled).into()
}
#[test]
fn chip_group_rejects_selection_mode_mismatches_and_unknown_or_duplicate_members() {
    let app = App::memory();
    for group in [
        chip_group("wrong-single")
            .values(vec!["a".to_owned()])
            .item(chip_item("a", "A")),
        chip_group("wrong-multi")
            .mode(ChipMode::Multiple)
            .value("a")
            .item(chip_item("a", "A")),
        chip_group("duplicate").items([chip_item("a", "A"), chip_item("a", "Again")]),
        chip_group("unknown")
            .default_value("missing")
            .item(chip_item("a", "A")),
    ] {
        assert!(app.mount(group).is_err());
        assert_eq!(app.stats().nodes, 0);
    }
}
#[test]
fn chip_groups_restore_rejected_multi_proposals_and_release_owner_captures() {
    let app = App::memory();
    let source = app.scope().cage(vec!["a".to_owned()]);
    let calls = std::rc::Rc::new(std::cell::Cell::new(0));
    let seen = calls.clone();
    app.mount(
        chip_group("controlled-multi")
            .mode(ChipMode::Multiple)
            .values(source)
            .items([chip_item("a", "A"), chip_item("b", "B")])
            .on_values_change(move |_| seen.set(seen.get() + 1)),
    )
    .unwrap();
    let members = find(&app, "sapid-checkbox-group-item");
    app.dispatch(members[1], Event::CheckedChanged(true))
        .unwrap();
    assert_eq!(source.get(), ["a"]);
    assert!(!checked(&app, members[1]));
    assert_eq!(calls.get(), 1);
    app.clear().unwrap();
    assert_eq!(app.stats().handlers, 0);
    assert_eq!(std::rc::Rc::strong_count(&calls), 1);
}
#[test]
fn chip_ssr_uses_native_choice_semantics_and_never_pressed_or_tab_states() {
    let snapshot = glory::ssr::snapshot_with(|_| {
        glory::ui::group().fill((
            chip().label("Featured").default_checked(true),
            chip_group("one")
                .default_value("a")
                .item(chip_item("a", "A")),
            chip_group("many")
                .mode(ChipMode::Multiple)
                .item(chip_item("b", "B")),
        ))
    })
    .unwrap();
    assert!(snapshot.html.contains("role=\"radiogroup\""));
    assert_eq!(snapshot.html.matches("type=\"checkbox\"").count(), 2);
    assert_eq!(snapshot.html.matches("type=\"radio\"").count(), 1);
    assert!(
        !snapshot.html.contains("aria-pressed")
            && !snapshot.html.contains("tablist")
            && !snapshot.html.contains("tabpanel")
    );
}
