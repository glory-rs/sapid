use glory::render::{Event, Navigation, Orientation, Props, Role};
use glory::ui::App;
use sapid::{segment, segmented_control};
#[test]
fn segmented_control_reuses_native_radio_selection_and_preserves_custom_classes() {
    let app = App::memory();
    let first = app.scope().cage(None);
    let second = app.scope().cage(None);
    let root = app.scope().cage(None);
    let value = app.scope().cage("monthly".to_owned());
    app.mount(
        segmented_control("billing")
            .label("Billing")
            .node_ref(root)
            .class("custom-group")
            .value(value)
            .segments([
                segment("monthly")
                    .label("Monthly")
                    .class("custom-segment")
                    .node_ref(first),
                segment("yearly").label("Yearly").node_ref(second),
            ]),
    )
    .unwrap();
    let first = first.get().unwrap();
    let second = second.get().unwrap();
    let root = root.get().unwrap();
    assert_eq!(
        app.tree().node(root).unwrap().descriptor.view().role,
        Some(Role::RadioGroup)
    );
    let classes = app
        .tree()
        .node(root)
        .unwrap()
        .descriptor
        .view()
        .classes
        .clone();
    assert!(
        classes
            .iter()
            .any(|class| class == "sapid-segmented-control")
            && classes.iter().any(|class| class == "custom-group")
    );
    let classes = app
        .tree()
        .node(first)
        .unwrap()
        .descriptor
        .view()
        .classes
        .clone();
    assert!(
        classes.iter().any(|class| class == "sapid-segment")
            && classes.iter().any(|class| class == "custom-segment")
    );
    assert_eq!(
        app.tree()
            .node(root)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .navigation,
        Some(Orientation::Horizontal)
    );
    app.dispatch(second, Event::CheckedChanged(true)).unwrap();
    assert_eq!(value.get(), "monthly");
    assert!(
        matches!(&app.tree().node(first).unwrap().descriptor.view().props.clone(),Props::Check(check) if check.checked)
    );
    assert!(
        matches!(&app.tree().node(second).unwrap().descriptor.view().props.clone(),Props::Check(check) if !check.checked)
    );
    app.dispatch(
        first,
        Event::Navigate {
            from: first,
            command: Navigation::Next,
        },
    )
    .unwrap();
    assert_eq!(value.get(), "monthly");
}
#[test]
fn segmented_local_selection_and_ssr_use_values_without_tab_panels() {
    let snapshot = glory::ssr::snapshot_with(|_| {
        segmented_control("period")
            .default_value("month")
            .segments([
                segment("month").label("Monthly"),
                segment("year").label("Yearly"),
            ])
    })
    .unwrap();
    assert!(snapshot.html.contains("role=\"radiogroup\""));
    assert_eq!(snapshot.html.matches("type=\"radio\"").count(), 2);
    assert!(!snapshot.html.contains("tabpanel") && !snapshot.html.contains("tablist"));
    let app = App::memory();
    let second = app.scope().cage(None);
    app.mount(
        segmented_control("period")
            .default_value("month")
            .segments([
                segment("month").label("Monthly"),
                segment("year").label("Yearly").node_ref(second),
            ]),
    )
    .unwrap();
    let second = second.get().unwrap();
    app.dispatch(second, Event::CheckedChanged(true)).unwrap();
    assert!(
        matches!(&app.tree().node(second).unwrap().descriptor.view().props.clone(),Props::Check(check) if check.checked)
    );
    app.clear().unwrap();
    assert_eq!(app.stats().nodes, 0);
    assert_eq!(app.stats().handlers, 0);
}
