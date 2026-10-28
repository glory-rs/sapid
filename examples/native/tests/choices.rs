use glory::blitz::shell::WindowApp;
use glory::html::Part;
use glory::render::{Event, NodeId, Query};
use glory::ui::App;
use sapid::{
    CheckState, ChipMode, chip, chip_group, chip_item, field, mixed_checkbox, radio,
    radio_group, segment, segmented_control,
};
fn find(app: &App, class: &str) -> Vec<NodeId> {
    let tree = app.tree();
    let mut pending = tree.roots().iter().rev().copied().collect::<Vec<_>>();
    let mut result = Vec::new();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if node
            .descriptor
            .view()
            .classes
            .iter()
            .any(|value| value == class)
        {
            result.push(id);
        }
        pending.extend(node.children.iter().rev().copied());
    }
    result
}
fn checked(host: &WindowApp, node: NodeId) -> bool {
    let physical = host.renderer().engine_id(node, Part::Control).unwrap();
    host.renderer().inspect(|document| {
        document
            .get_node(physical)
            .unwrap()
            .element_data()
            .unwrap()
            .checkbox_input_checked()
            .unwrap()
    })
}
#[test]
fn standalone_and_multiple_chips_keep_real_native_checkbox_values_field_and_identity() {
    let host = WindowApp::new();
    let source = host.app().scope().cage(Vec::<String>::new());
    let readonly = host.app().scope().cage(false);
    host.mount(
        field("chips-native")
            .label("Interests")
            .required(true)
            .readonly(readonly)
            .control(move |field| {
                chip_group("interests-native")
                    .mode(ChipMode::Multiple)
                    .field(field)
                    .values(source)
                    .items([chip_item("a", "A"), chip_item("b", "B")])
                    .on_values_change(move |value| source.revise(|old| *old = value))
            }),
    )
    .unwrap();
    let members = find(host.app(), "sapid-checkbox-group-item");
    host.app()
        .request(Query::Bounds(members[0]), |result| {
            result.unwrap();
        })
        .unwrap();
    host.app().pump().unwrap();
    let identity = host
        .renderer()
        .engine_id(members[1], Part::Control)
        .unwrap();
    assert!(!checked(&host, members[0]) && !checked(&host, members[1]));
    host.app()
        .dispatch(members[1], Event::CheckedChanged(true))
        .unwrap();
    host.app().settle(8).unwrap();
    assert!(!checked(&host, members[0]) && checked(&host, members[1]));
    assert_eq!(
        host.renderer()
            .engine_id(members[1], Part::Control)
            .unwrap(),
        identity
    );
    readonly.revise(|value| *value = true);
    host.app().flush().unwrap();
    assert!(
        !host
            .app()
            .dispatch(members[0], Event::CheckedChanged(true))
            .unwrap()
    );
    assert!(!checked(&host, members[0]));
    host.app().clear().unwrap();
    assert!(host.app().tree().is_empty());
    host.mount(chip().label("Featured").default_checked(true))
        .unwrap();
    let node = find(host.app(), "sapid-chip")[0];
    host.app()
        .request(Query::Bounds(node), |result| {
            result.unwrap();
        })
        .unwrap();
    host.app().pump().unwrap();
    assert!(checked(&host, node));
    host.app().clear().unwrap();
}
#[test]
fn native_mixed_and_single_value_variants_preserve_real_controls_and_controlled_state() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        let app = host.app();
        let state = app.scope().cage(CheckState::Mixed);
        host.mount(
            mixed_checkbox()
                .theme(theme.clone())
                .state(state)
                .on_state_change(move |value| state.revise(|old| *old = value)),
        )
        .unwrap();
        app.settle(8).unwrap();
        let node = find(app, "sapid-mixed-checkbox")[0];
        let physical = host.renderer().engine_id(node, Part::Control).unwrap();
        assert!(host.renderer().inspect(|doc| {
            doc.get_node(physical)
                .unwrap()
                .element_data()
                .unwrap()
                .checkbox_input_indeterminate()
                .unwrap()
        }));
        app.dispatch(node, Event::CheckedChanged(true)).unwrap();
        app.settle(8).unwrap();
        assert_eq!(state.get(), CheckState::Checked);
        assert!(checked(&host, node));
        assert!(!host.renderer().inspect(|doc| {
            doc.get_node(physical)
                .unwrap()
                .element_data()
                .unwrap()
                .checkbox_input_indeterminate()
                .unwrap()
        }));
        app.clear().unwrap();
        for variant in 0..3 {
            let value = app.scope().cage("a".to_owned());
            let readonly = app.scope().cage(false);
            let widget: Box<dyn glory::Widget> = match variant {
                0 => Box::new(
                    radio_group("native-radio")
                        .theme(theme.clone())
                        .value(value)
                        .readonly(readonly)
                        .radio(radio("a").label("A"))
                        .radio(radio("b").label("B"))
                        .on_value_change(move |next| value.revise(|old| *old = next)),
                ),
                1 => Box::new(
                    segmented_control("native-segments")
                        .theme(theme.clone())
                        .value(value)
                        .readonly(readonly)
                        .segment(segment("a").label("A"))
                        .segment(segment("b").label("B"))
                        .on_value_change(move |next| value.revise(|old| *old = next)),
                ),
                _ => Box::new(
                    chip_group("native-single")
                        .theme(theme.clone())
                        .value(value)
                        .readonly(readonly)
                        .items([chip_item("a", "A"), chip_item("b", "B")])
                        .on_value_change(move |next| value.revise(|old| *old = next)),
                ),
            };
            host.mount(widget).unwrap();
            app.settle(8).unwrap();
            let members = find(app, "sapid-radio");
            assert_eq!(members.len(), 2);
            let identity = host
                .renderer()
                .engine_id(members[1], Part::Control)
                .unwrap();
            assert!(checked(&host, members[0]) && !checked(&host, members[1]));
            app.dispatch(members[1], Event::CheckedChanged(true))
                .unwrap();
            app.settle(8).unwrap();
            assert_eq!(value.get(), "b");
            assert!(!checked(&host, members[0]) && checked(&host, members[1]));
            readonly.revise(|old| *old = true);
            app.flush().unwrap();
            assert!(
                !app.dispatch(members[0], Event::CheckedChanged(true))
                    .unwrap()
            );
            assert_eq!(value.get(), "b");
            assert_eq!(
                host.renderer()
                    .engine_id(members[1], Part::Control)
                    .unwrap(),
                identity
            );
            app.clear().unwrap();
            assert!(app.tree().is_empty());
        }
    }
}
