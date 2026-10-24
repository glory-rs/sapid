use glory::render::{CheckMode, Event, EventKind, FocusMode, Navigation, NodeId, Props, Role};
use glory::ui::{App, Widget};
use sapid::{field, radio, radio_group};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
fn radios(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut pending = tree.roots().iter().rev().copied().collect::<Vec<_>>();
    let mut result = Vec::new();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if matches!(&node.descriptor.view().props.clone(), Props::Check(check) if check.mode == CheckMode::Radio)
        {
            result.push(id);
        }
        pending.extend(node.children.iter().rev().copied());
    }
    result
}
fn checked(app: &App) -> Vec<bool> {
    radios(app).into_iter().map(|id| matches!(&app.tree().node(id).unwrap().descriptor.view().props.clone(), Props::Check(check) if check.checked)).collect()
}
fn navigate(app: &App, node: NodeId, command: Navigation) {
    app.dispatch(
        node,
        Event::Navigate {
            from: node,
            command,
        },
    )
    .unwrap();
}
#[test]
fn radio_group_has_real_semantics_one_value_and_a_single_roving_tab_stop() {
    let app = App::memory();
    let value = app.scope().cage("one".to_string());
    app.mount(
        radio_group("delivery")
            .label("Delivery")
            .form_name("delivery-choice")
            .value(value)
            .items([
                radio("one").label("One"),
                radio("two").label("Two").enabled(false),
                radio("three").label("Three"),
            ])
            .on_value_change(move |next| value.revise(|old| *old = next)),
    )
    .unwrap();
    let group = app.tree().roots()[0];
    assert_eq!(
        app.tree().node(group).unwrap().descriptor.view().role,
        Some(Role::RadioGroup)
    );
    let members = radios(&app);
    assert_eq!(checked(&app), [true, false, false]);
    navigate(&app, members[0], Navigation::Next);
    assert_eq!(value.get(), "three");
    assert_eq!(checked(&app), [false, false, true]);
    assert_eq!(
        members
            .iter()
            .filter(|id| app.tree().node(**id).unwrap().descriptor.view().focus
                == FocusMode::TabStop)
            .count(),
        1
    );
    assert_eq!(
        app.tree().node(members[2]).unwrap().descriptor.view().focus,
        FocusMode::TabStop
    );
    navigate(&app, members[2], Navigation::Next);
    assert_eq!(value.get(), "one");
    for member in members {
        let tree = app.tree();
        let descriptor = tree.node(member).unwrap().descriptor.view();
        let metadata = descriptor.semantics.as_ref().unwrap();
        assert_eq!(
            metadata.check_group.as_ref().unwrap().as_str(),
            "delivery-choice"
        );
        assert_eq!(
            metadata.form_control.as_ref().unwrap().name,
            "delivery-choice"
        );
        assert!(metadata.navigation_item);
    }
}
#[test]
fn rejection_reasserts_the_complete_radio_group_without_writing_controlled_value() {
    let app = App::memory();
    let value = app.scope().cage("one".to_owned());
    let calls = Rc::new(RefCell::new(Vec::new()));
    let seen = calls.clone();
    app.mount(
        radio_group("controlled")
            .value(value)
            .default_value("two")
            .items([radio("one").label("One"), radio("two").label("Two")])
            .on_value_change(move |value| seen.borrow_mut().push(value)),
    )
    .unwrap();
    let members = radios(&app);
    app.dispatch(members[1], Event::CheckedChanged(true))
        .unwrap();
    assert_eq!(value.get(), "one");
    assert_eq!(checked(&app), [true, false]);
    navigate(&app, members[0], Navigation::Next);
    assert_eq!(value.get(), "one");
    assert_eq!(checked(&app), [true, false]);
    assert_eq!(*calls.borrow(), ["two", "two"]);
    // Native hydration reports an implicitly unchecked peer as false; it cannot
    // clear the group's value or deliver a second change proposal.
    app.dispatch(members[0], Event::CheckedChanged(false))
        .unwrap();
    assert_eq!(calls.borrow().len(), 2);
    value.revise(|value| *value = "two".into());
    app.flush().unwrap();
    assert_eq!(checked(&app), [false, true]);
    assert_eq!(radios(&app), members);
}
#[test]
fn radio_field_metadata_readonly_and_disabled_are_shared_with_native_members() {
    let app = App::memory();
    let readonly = app.scope().cage(false);
    let enabled = app.scope().cage(true);
    app.mount(
        field("contact")
            .label("Contact method")
            .description("Choose one")
            .required(true)
            .invalid(true)
            .readonly(readonly)
            .enabled(enabled)
            .control(|field| {
                radio_group("contact-choice")
                    .field(field)
                    .default_value("email")
                    .items([radio("email").label("Email"), radio("phone").label("Phone")])
            }),
    )
    .unwrap();
    let members = radios(&app);
    for member in &members {
        let descriptor = app
            .tree()
            .node(*member)
            .unwrap()
            .descriptor
            .view()
            .to_owned();
        let metadata = descriptor.semantics.unwrap();
        assert!(metadata.required && metadata.invalid);
    }
    readonly.revise(|value| *value = true);
    app.flush().unwrap();
    assert!(
        !app.dispatch(members[1], Event::CheckedChanged(true))
            .unwrap()
    );
    assert_eq!(checked(&app), [true, false]);
    enabled.revise(|value| *value = false);
    app.flush().unwrap();
    assert!(
        !app.dispatch(members[1], Event::CheckedChanged(true))
            .unwrap()
    );
    assert_eq!(checked(&app), [true, false]);
}
#[test]
fn disabled_selected_value_is_preserved_with_an_enabled_tab_entry_and_nonlooping_edges() {
    let app = App::memory();
    app.mount(
        radio_group("disabled-choice")
            .default_value("two")
            .loop_focus(false)
            .items([
                radio("one").label("One"),
                radio("two").label("Two").enabled(false),
                radio("three").label("Three"),
            ]),
    )
    .unwrap();
    let members = radios(&app);
    assert_eq!(checked(&app), [false, true, false]);
    assert_eq!(
        app.tree().node(members[0]).unwrap().descriptor.view().focus,
        FocusMode::TabStop
    );
    navigate(&app, members[0], Navigation::Previous);
    assert_eq!(checked(&app), [false, true, false]);
    navigate(&app, members[0], Navigation::Last);
    assert_eq!(checked(&app), [false, false, true]);
}
#[test]
fn invalid_radio_configurations_are_rejected_before_nodes_and_program_unknown_values_preserve_ui()
 {
    let app = App::memory();
    for group in [
        radio_group("empty"),
        radio_group("duplicate").items([radio("a"), radio("a")]),
        radio_group("unknown").radio(radio("a")).default_value("b"),
        radio_group("bad-name")
            .radio(radio("a"))
            .form_name("business name with spaces"),
        radio_group("budget").items(
            (0..=sapid::MAX_RADIO_GROUP_ITEMS).map(|index| radio(format!("item{index}"))),
        ),
    ] {
        assert!(app.mount(group).is_err());
        assert_eq!(app.stats().nodes, 0);
    }
    let value = app.scope().cage("a".to_owned());
    app.mount(
        radio_group("updates")
            .value(value)
            .items([radio("a"), radio("b")]),
    )
    .unwrap();
    value.revise(|value| *value = "missing".into());
    assert!(app.flush().is_err());
    assert_eq!(checked(&app), [true, false]);
    value.revise(|value| *value = "b".into());
    app.flush().unwrap();
    assert_eq!(checked(&app), [false, true]);
}
#[test]
fn radio_handlers_stop_after_owner_retirement_and_release_captures() {
    let app = App::memory();
    let count = Rc::new(Cell::new(0));
    let owner = app.scope().child().unwrap();
    let retire = owner.clone();
    let seen = count.clone();
    radio_group("retire")
        .items([radio("a").on(EventKind::CheckedChanged, |_| {
            panic!("retired member observer")
        })])
        .on_value_change(move |_| {
            seen.set(seen.get() + 1);
            retire.dispose();
        })
        .show_in(&owner)
        .unwrap();
    app.flush().unwrap();
    let node = radios(&app)[0];
    app.dispatch(node, Event::CheckedChanged(true)).unwrap();
    assert_eq!(count.get(), 1);
    assert_eq!(Rc::strong_count(&count), 1);
    assert_eq!(app.stats().nodes, 0);
    assert_eq!(app.stats().handlers, 0);
}
#[test]
fn radio_ssr_is_a_true_named_native_group_and_successful_values_are_explicit() {
    let snapshot = glory::ssr::snapshot_with(|_| {
        radio_group("shipping")
            .label("Shipping")
            .form_name("shipping")
            .default_value("express")
            .items([
                radio("standard").label("Standard"),
                radio("express").label("Express"),
            ])
    })
    .unwrap();
    assert!(snapshot.html.contains("role=\"radiogroup\""));
    assert_eq!(snapshot.html.matches("type=\"radio\"").count(), 2);
    assert_eq!(snapshot.html.matches("name=\"shipping\"").count(), 2);
    assert!(snapshot.html.contains("value=\"express\""));
    assert_eq!(snapshot.html.matches("tabindex=\"0\"").count(), 1);
}
