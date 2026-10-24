use glory::prelude::*;
use glory::render::{CheckMode, EventKind, FocusMode, NodeId, Props};
use sapid::{RatingPrecision, field, rating};
use std::{cell::RefCell, rc::Rc};

fn radios(app: &App) -> Vec<NodeId> {
    let tree = app.tree();
    let mut result = Vec::new();
    let mut pending: Vec<_> = tree.roots().iter().rev().copied().collect();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if matches!(node.descriptor.view().props, Props::Check(p) if p.mode==CheckMode::Radio) {
            result.push(id)
        }
        pending.extend(node.children.iter().rev().copied());
    }
    result
}
fn checked(app: &App, members: &[NodeId]) -> Vec<bool> {
    let tree = app.tree();
    members
        .iter()
        .map(|id| {
            let Props::Check(props) = tree.node(*id).unwrap().descriptor.view().props else {
                panic!("radio")
            };
            props.checked
        })
        .collect()
}
#[test]
fn half_values_are_real_radios_with_one_selection_and_zero_gap_between_halves() {
    let app = App::memory();
    app.mount(
        rating("half")
            .precision(RatingPrecision::Half)
            .default_value(1.5),
    )
    .unwrap();
    let members = radios(&app);
    assert_eq!(members.len(), 10);
    assert_eq!(
        checked(&app, &members),
        [
            false, false, true, false, false, false, false, false, false, false
        ]
    );
    let tree = app.tree();
    let root = tree.roots()[0];
    let group = tree.node(root).unwrap();
    assert_eq!(
        group.descriptor.view().role,
        Some(glory::render::Role::RadioGroup)
    );
    let items = tree.node(group.children[1]).unwrap();
    assert_eq!(items.descriptor.view().style.gap, Some(0.));
    for (index, id) in members.iter().enumerate() {
        let view = tree.node(*id).unwrap().descriptor.view();
        assert_eq!(
            view.name.as_deref(),
            Some(format!("{} of 5", (index + 1) as f64 / 2.).as_str())
        );
    }
    assert_eq!(
        members
            .iter()
            .filter(|id| tree.node(**id).unwrap().descriptor.view().focus == FocusMode::TabStop)
            .count(),
        1
    );
}
#[test]
fn local_selection_and_explicit_clear_report_exact_values_before_generic_observers() {
    let app = App::memory();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let changed = calls.clone();
    let generic = calls.clone();
    app.mount(
        rating("local")
            .precision(RatingPrecision::Half)
            .clearable(true)
            .on_value_change(move |v| changed.borrow_mut().push(format!("change:{v}")))
            .on(EventKind::CheckedChanged, move |_| {
                generic.borrow_mut().push("generic".into())
            }),
    )
    .unwrap();
    let members = radios(&app);
    assert_eq!(members.len(), 11);
    assert!(checked(&app, &members)[0]);
    app.dispatch(members[3], Event::CheckedChanged(true))
        .unwrap();
    assert!(checked(&app, &members)[3]);
    assert_eq!(calls.borrow()[0], "change:1.5");
    app.dispatch(members[0], Event::CheckedChanged(true))
        .unwrap();
    assert!(checked(&app, &members)[0]);
    assert_eq!(
        &*calls.borrow(),
        &["change:1.5", "generic", "change:0", "generic"]
    );
}
#[test]
fn controlled_rejection_and_invalid_recovery_retain_the_accepted_radio() {
    let app = App::memory();
    let value = app.scope().cage(2.5);
    let request = app.scope().cage(0.);
    app.mount(
        rating("controlled")
            .precision(RatingPrecision::Half)
            .value(value)
            .on_value_change(move |next| request.revise(|v| *v = next)),
    )
    .unwrap();
    let members = radios(&app);
    app.dispatch(members[7], Event::CheckedChanged(true))
        .unwrap();
    assert_eq!(request.get(), 4.);
    assert_eq!(value.get(), 2.5);
    assert!(checked(&app, &members)[4]);
    value.revise(|v| *v = 2.25);
    assert!(app.flush().is_err());
    assert!(checked(&app, &members)[4]);
    value.revise(|v| *v = 4.5);
    app.flush().unwrap();
    assert!(checked(&app, &members)[8]);
    assert_eq!(request.get(), 4., "external updates must be silent");
}
#[test]
fn readonly_and_disabled_field_metadata_stay_on_real_radio_controls() {
    let app = App::memory();
    let readonly = app.scope().cage(true);
    let enabled = app.scope().cage(true);
    app.mount(
        field("rating")
            .label("Quality")
            .description("Choose a quality score")
            .readonly(readonly)
            .enabled(enabled)
            .required(true)
            .control(|context| {
                rating("quality")
                    .field(context)
                    .form_name("quality")
                    .default_value(3.)
            }),
    )
    .unwrap();
    let members = radios(&app);
    {
        let tree = app.tree();
        let root = tree.node(tree.roots()[0]).unwrap();
        assert!(!root.children.is_empty());
        for id in &members {
            let view = tree.node(*id).unwrap().descriptor.view();
            let metadata = view.semantics.as_ref().unwrap();
            assert!(metadata.required && metadata.readonly);
            assert_eq!(metadata.form_control.as_ref().unwrap().name, "quality");
        }
    }
    assert!(
        !app.dispatch(members[3], Event::CheckedChanged(true))
            .unwrap()
    );
    assert!(checked(&app, &members)[2]);
    readonly.revise(|v| *v = false);
    app.flush().unwrap();
    app.dispatch(members[3], Event::CheckedChanged(true))
        .unwrap();
    assert!(checked(&app, &members)[3]);
    enabled.revise(|v| *v = false);
    app.flush().unwrap();
    for id in &members {
        let tree = app.tree();
        let Props::Check(props) = tree.node(*id).unwrap().descriptor.view().props else {
            panic!()
        };
        assert!(!props.enabled);
    }
}
#[test]
fn invalid_count_precision_values_and_names_create_no_nodes() {
    for widget in [
        rating("bad").count(0),
        rating("bad").count(11),
        rating("bad").count(255),
        rating("bad").default_value(0.5),
        rating("bad").default_value(6.),
        rating("bad").default_value(f64::NAN),
        rating("bad")
            .precision(RatingPrecision::Half)
            .default_value(0.25),
        rating("bad").name("x".repeat(1_000_000)),
    ] {
        let app = App::memory();
        assert!(app.mount(widget).is_err());
        assert!(app.tree().is_empty());
    }
}
#[test]
fn maximum_half_star_count_is_bounded_and_negative_zero_matches_the_clear_radio() {
    let app = App::memory();
    app.mount(
        rating("max")
            .count(10)
            .precision(RatingPrecision::Half)
            .clearable(true)
            .default_value(-0.),
    )
    .unwrap();
    let members = radios(&app);
    assert_eq!(members.len(), 21);
    assert_eq!(checked(&app, &members).iter().filter(|v| **v).count(), 1);
    assert!(checked(&app, &members)[0]);
}
#[test]
fn ssr_uses_native_radio_inputs_and_preserves_half_value_names() {
    let html = glory::ssr::render(
        rating("ssr")
            .precision(RatingPrecision::Half)
            .form_name("quality")
            .default_value(2.5)
            .readonly(true),
    )
    .unwrap();
    assert_eq!(html.matches("type=\"radio\"").count(), 10);
    assert!(html.contains("role=\"radiogroup\""));
    assert!(html.contains("value=\"2.5\""));
    assert!(html.contains("aria-label=\"2.5 of 5\""));
    assert!(html.contains("name=\"quality\""));
    assert!(!html.contains("<button"));
    assert_eq!(html.matches("<svg").count(), 10);
}
#[test]
fn owner_retirement_releases_wrapped_radios_and_public_reference() {
    let app = App::memory();
    let reference = app.scope().cage(None);
    app.mount(
        rating("owner")
            .precision(RatingPrecision::Half)
            .node_ref(reference),
    )
    .unwrap();
    let old = radios(&app);
    app.clear().unwrap();
    assert!(reference.get().is_none());
    assert!(app.tree().is_empty());
    app.mount(rating("owner").node_ref(reference)).unwrap();
    let fresh = radios(&app);
    assert!(old.iter().all(|id| !fresh.contains(id)));
}

#[test]
fn display_mode_is_a_named_passive_group_with_no_radios_or_form_registration() {
    let app = App::memory();
    let value = app.scope().cage(2.5);
    let reference = app.scope().cage(None);
    app.mount(
        rating("display")
            .display_only()
            .precision(RatingPrecision::Half)
            .name("Quality")
            .value(value)
            .node_ref(reference),
    )
    .unwrap();
    assert!(radios(&app).is_empty());
    let id = reference.get().unwrap();
    assert_eq!(
        app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref(),
        Some("Quality: 2.5 of 5")
    );
    value.revise(|v| *v = 4.);
    app.flush().unwrap();
    assert_eq!(
        app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref(),
        Some("Quality: 4 of 5")
    );
    let html = glory::ssr::render(
        rating("display")
            .display_only()
            .precision(RatingPrecision::Half)
            .default_value(2.5),
    )
    .unwrap();
    assert!(html.contains("role=\"group\""));
    assert!(!html.contains("<input"));
    assert!(!html.contains("name=\""));
    assert!(!html.contains("aria-checked"));
    assert_eq!(html.matches("<svg").count(), 10);
}
#[test]
fn display_mode_explicitly_rejects_form_names_and_change_handlers() {
    for widget in [
        rating("display").display_only().form_name("score"),
        rating("display").display_only().on_value_change(|_| {}),
    ] {
        let app = App::memory();
        assert!(app.mount(widget).is_err());
        assert!(app.tree().is_empty());
    }
}

#[test]
fn wrapped_members_keep_radio_navigation_and_visible_focus_within_decoration() {
    use glory::render::Navigation;
    use glory::style::Edges;
    let app = App::memory();
    app.mount(
        rating("keys")
            .precision(RatingPrecision::Half)
            .clearable(true)
            .default_value(2.5),
    )
    .unwrap();
    let members = radios(&app);
    let node = members[5];
    app.dispatch(node, Event::FocusChanged(true)).unwrap();
    {
        let tree = app.tree();
        let parent = tree.node(tree.node(node).unwrap().parent.unwrap()).unwrap();
        let indicator = parent
            .children
            .iter()
            .find_map(|id| {
                let node = tree.node(*id).unwrap();
                node.descriptor
                    .view()
                    .classes
                    .iter()
                    .any(|v| v == "sapid-rating-focus")
                    .then_some(node)
            })
            .unwrap();
        assert_eq!(
            indicator.descriptor.view().style.border_width,
            Some(Edges::all(2.))
        );
    }
    app.dispatch(
        node,
        Event::Navigate {
            from: node,
            command: Navigation::Next,
        },
    )
    .unwrap();
    assert!(checked(&app, &members)[6]);
    app.dispatch(
        members[6],
        Event::Navigate {
            from: members[6],
            command: Navigation::First,
        },
    )
    .unwrap();
    assert!(checked(&app, &members)[0]);
    app.dispatch(
        members[0],
        Event::Navigate {
            from: members[0],
            command: Navigation::Last,
        },
    )
    .unwrap();
    assert!(checked(&app, &members)[10]);
}
