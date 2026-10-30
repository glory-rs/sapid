use glory::ui::App;
use sapid::{
    checkbox_group, checkbox_item, chip_group, chip_item, radio, radio_group, segment,
    segmented_control,
};
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn choices_receive_at_most_budget_plus_one_items_from_large_iterators() {
    let app = App::memory();
    let reads = Rc::new(Cell::new(0));
    let seen = reads.clone();
    let widget = radio_group("bounded-radio").items((0..1_000_000).map(move |index| {
        seen.set(seen.get() + 1);
        radio(format!("r{index}"))
    }));
    assert_eq!(reads.get(), 1025);
    assert!(app.mount(widget).is_err());
    assert_eq!(app.stats().nodes, 0);
    reads.set(0);
    let seen = reads.clone();
    let widget = checkbox_group("bounded-check").items((0..1_000_000).map(move |index| {
        seen.set(seen.get() + 1);
        checkbox_item(format!("c{index}"), "Check")
    }));
    assert_eq!(reads.get(), 1025);
    assert!(app.mount(widget).is_err());
    assert_eq!(app.stats().nodes, 0);
    reads.set(0);
    let seen = reads.clone();
    let widget = segmented_control("bounded-segment").segments((0..1_000_000).map(move |index| {
        seen.set(seen.get() + 1);
        segment(format!("s{index}"))
    }));
    assert_eq!(reads.get(), 1025);
    assert!(app.mount(widget).is_err());
    assert_eq!(app.stats().nodes, 0);
    reads.set(0);
    let seen = reads.clone();
    let widget = chip_group("bounded-chip").items((0..1_000_000).map(move |index| {
        seen.set(seen.get() + 1);
        chip_item(format!("p{index}"), "Chip")
    }));
    assert_eq!(reads.get(), 1025);
    assert!(app.mount(widget).is_err());
    assert_eq!(app.stats().nodes, 0);
    let mut radio = radio_group("incremental-radio");
    let mut check = checkbox_group("incremental-check");
    let mut segments = segmented_control("incremental-segments");
    let mut chips = chip_group("incremental-chips");
    for index in 0..10_000 {
        radio = radio.radio(sapid::radio(format!("r{index}")));
        check = check.item(checkbox_item(format!("c{index}"), "Check"));
        segments = segments.segment(segment(format!("s{index}")));
        chips = chips.item(chip_item(format!("p{index}"), "Chip"));
    }
    for failed in [
        app.mount(radio),
        app.mount(check),
        app.mount(segments),
        app.mount(chips),
    ] {
        assert!(failed.is_err());
    }
    assert_eq!(app.stats().nodes, 0);
}

#[test]
fn external_multiselection_is_validated_by_borrow_before_cloning_or_proposing() {
    let app = App::memory();
    let source = app.scope().cage(vec![String::new(); 1_000_000]);
    assert!(
        app.mount(
            checkbox_group("external-check")
                .selected(source)
                .item(checkbox_item("a", "A"))
        )
        .is_err()
    );
    assert!(
        app.mount(
            chip_group("external-chip")
                .mode(sapid::ChipMode::Multiple)
                .values(source)
                .item(chip_item("a", "A"))
        )
        .is_err()
    );
    assert_eq!(app.stats().nodes, 0);
    source.revise(|value| value.clear());
    let calls = Rc::new(Cell::new(0));
    let seen = calls.clone();
    app.mount(
        checkbox_group("late-external-check")
            .selected(source)
            .select_all_label("All")
            .item(checkbox_item("a", "A"))
            .on_selected_change(move |_| seen.set(seen.get() + 1)),
    )
    .unwrap();
    source.revise(|value| *value = vec![String::new(); 1_000_000]);
    assert!(app.flush().is_err());
    let ids = {
        let tree = app.tree();
        let mut pending = tree.roots().to_vec();
        let mut controls = Vec::new();
        while let Some(id) = pending.pop() {
            let node = tree.node(id).unwrap();
            if matches!(node.descriptor.view().props, glory::render::Props::Check(_)) {
                controls.push(id);
            }
            pending.extend(node.children.iter().copied());
        }
        controls
    };
    for id in ids {
        assert!(
            app.dispatch(id, glory::render::Event::CheckedChanged(true))
                .is_err()
        );
    }
    assert_eq!(calls.get(), 0);
    assert_eq!(source.with(Vec::len), 1_000_000);
}

#[test]
fn oversized_owned_constant_vec_and_string_are_rejected_at_the_borrowed_entry() {
    let app = App::memory();
    assert!(
        app.mount(
            checkbox_group("constant-check")
                .selected(vec![String::new(); 1_000_000])
                .item(checkbox_item("a", "A"))
        )
        .is_err()
    );
    assert!(
        app.mount(
            chip_group("constant-chip")
                .mode(sapid::ChipMode::Multiple)
                .values(vec![String::new(); 1_000_000])
                .item(chip_item("a", "A"))
        )
        .is_err()
    );
    assert!(
        app.mount(
            radio_group("constant-radio")
                .value("x".repeat(1_000_000))
                .radio(radio("a").label("A"))
        )
        .is_err()
    );
    assert_eq!(app.stats().nodes, 0);
}
