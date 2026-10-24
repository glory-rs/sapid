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
fn native_mixed_and_single_value_radio_variants_refuse_without_leaving_partial_controls() {
    let host = WindowApp::new();
    assert!(
        host.mount(mixed_checkbox().default_state(CheckState::Mixed))
            .is_err()
    );
    assert!(host.app().tree().is_empty());
    assert!(
        host.mount(radio_group("radio-native").radio(radio("a").label("A")))
            .is_err()
    );
    assert!(host.app().tree().is_empty());
    assert!(
        host.mount(segmented_control("segment-native").segment(segment("a").label("A")))
            .is_err()
    );
    assert!(host.app().tree().is_empty());
    assert!(
        host.mount(chip_group("single-native").item(chip_item("a", "A")))
            .is_err()
    );
    assert!(host.app().tree().is_empty());
}
