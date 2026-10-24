use glory::{
    Cage,
    ui::{App, Widget, widget},
};
use sapid::{
    button, button_group, data_list, data_list_item, grid, grid_item, highlight, text_list,
    text_list_item, timeline, timeline_item,
};
use std::{cell::Cell, rc::Rc};
fn reject(widget: impl Widget) {
    let app = App::memory();
    assert!(app.mount(widget).is_err());
    assert_eq!(app.stats().nodes, 0);
}
#[test]
fn million_item_iterators_consume_only_the_configuration_budget_plus_one() {
    let seen = Rc::new(Cell::new(0));
    let count = seen.clone();
    let widget = grid("budget").items((0..1_000_000).map(move |i| {
        count.set(count.get() + 1);
        grid_item(format!("item-{i}"))
    }));
    assert_eq!(seen.get(), 257);
    reject(widget);
    let seen = Rc::new(Cell::new(0));
    let count = seen.clone();
    let widget = timeline("budget").items((0..1_000_000).map(move |i| {
        count.set(count.get() + 1);
        timeline_item(format!("item-{i}"))
    }));
    assert_eq!(seen.get(), 257);
    reject(widget);
    let seen = Rc::new(Cell::new(0));
    let count = seen.clone();
    let widget = text_list().items((0..1_000_000).map(move |_| {
        count.set(count.get() + 1);
        text_list_item()
    }));
    assert_eq!(seen.get(), 257);
    reject(widget);
    let seen = Rc::new(Cell::new(0));
    let count = seen.clone();
    let widget = data_list().items((0..1_000_000).map(move |_| {
        count.set(count.get() + 1);
        data_list_item()
    }));
    assert_eq!(seen.get(), 257);
    reject(widget);
    let seen = Rc::new(Cell::new(0));
    let count = seen.clone();
    let widget = button_group().buttons((0..1_000_000).map(move |_| {
        count.set(count.get() + 1);
        button()
    }));
    assert_eq!(seen.get(), 129);
    reject(widget);
    let seen = Rc::new(Cell::new(0));
    let count = seen.clone();
    let widget = highlight()
        .content("x")
        .queries((0..1_000_000).map(move |_| {
            count.set(count.get() + 1);
            "x".to_string()
        }));
    assert_eq!(seen.get(), 33);
    reject(widget);
}
#[test]
fn repeated_items_setters_share_one_allowance_and_ignore_already_over_budget_iterators() {
    let seen = Rc::new(Cell::new(0));
    let count = seen.clone();
    let widget = grid("combined")
        .item(grid_item("one"))
        .items((0..1_000_000).map(move |i| {
            count.set(count.get() + 1);
            grid_item(format!("item-{i}"))
        }));
    assert_eq!(seen.get(), 256);
    let never = Rc::new(Cell::new(0));
    let count = never.clone();
    let widget = widget.items((0..1_000_000).map(move |_| {
        count.set(count.get() + 1);
        grid_item("unused")
    }));
    assert_eq!(never.get(), 0);
    reject(widget);
    let seen = Rc::new(Cell::new(0));
    let count = seen.clone();
    let widget = timeline("combined")
        .item(timeline_item("one"))
        .items((0..1_000_000).map(move |i| {
            count.set(count.get() + 1);
            timeline_item(format!("item-{i}"))
        }));
    assert_eq!(seen.get(), 256);
    reject(widget);
    let seen = Rc::new(Cell::new(0));
    let count = seen.clone();
    let widget = grid("tracks").column_tracks((0..1_000_000).map(move |_| {
        count.set(count.get() + 1);
        glory::style::GridTrack::Fr(1.0)
    }));
    assert_eq!(seen.get(), 65);
    reject(widget);
    let seen = Rc::new(Cell::new(0));
    let count = seen.clone();
    let widget = grid("rows").row_tracks((0..1_000_000).map(move |_| {
        count.set(count.get() + 1);
        glory::style::GridTrack::Auto
    }));
    assert_eq!(seen.get(), 65);
    reject(widget);
}
struct Capture {
    live: Rc<Cell<usize>>,
    drops: Cage<usize>,
}
impl Drop for Capture {
    fn drop(&mut self) {
        self.live.set(self.live.get() - 1);
        self.drops
            .try_revise(|drops| *drops += 1)
            .expect("configuration capture drop must not hold a graph borrow");
    }
}
fn content(live: &Rc<Cell<usize>>, drops: Cage<usize>) -> impl Widget {
    live.set(live.get() + 1);
    let capture = Capture {
        live: live.clone(),
        drops,
    };
    widget(move |_| {
        let _ = &capture;
        Ok(())
    })
}
#[test]
fn repeated_single_item_builders_keep_bounded_captures_and_drop_excess_outside_graph_borrows() {
    let app = App::memory();
    let drops = app.scope().cage(0usize);
    let live = Rc::new(Cell::new(0));
    let mut widget = grid("single");
    for i in 0..4096 {
        widget = widget.item(grid_item(format!("item-{i}")).fill(content(&live, drops)));
    }
    assert_eq!(live.get(), 257);
    assert_eq!(drops.get(), 4096 - 257);
    assert!(app.mount(widget).is_err());
    assert_eq!(live.get(), 0);
    assert_eq!(drops.get(), 4096);
    let mut widget = timeline("single");
    for i in 0..4096 {
        widget = widget.item(timeline_item(format!("item-{i}")).fill(content(&live, drops)));
    }
    assert_eq!(live.get(), 257);
    assert!(app.mount(widget).is_err());
    assert_eq!(live.get(), 0);
    let mut widget = text_list();
    for _ in 0..4096 {
        widget = widget.item(text_list_item().fill(content(&live, drops)));
    }
    assert_eq!(live.get(), 257);
    assert!(app.mount(widget).is_err());
    assert_eq!(live.get(), 0);
    let mut widget = data_list();
    for _ in 0..4096 {
        widget = widget.item(data_list_item().fill(content(&live, drops)));
    }
    assert_eq!(live.get(), 257);
    assert!(app.mount(widget).is_err());
    assert_eq!(live.get(), 0);
    assert_eq!(drops.get(), 4 * 4096);
    let mut buttons = button_group();
    for _ in 0..4096 {
        live.set(live.get() + 1);
        let capture = Capture {
            live: live.clone(),
            drops,
        };
        buttons = buttons.button(button().on_press(move || {
            let _ = &capture;
        }));
    }
    assert_eq!(live.get(), 129);
    assert!(app.mount(buttons).is_err());
    assert_eq!(live.get(), 0);
    assert_eq!(drops.get(), 5 * 4096);
    let mut marked = highlight().content("x");
    for _ in 0..4096 {
        marked = marked.query("x");
    }
    let seen = Rc::new(Cell::new(0));
    let count = seen.clone();
    let marked = marked.queries((0..1_000_000).map(move |_| {
        count.set(count.get() + 1);
        "x".into()
    }));
    assert_eq!(seen.get(), 0);
    reject(marked);
}
#[test]
fn maximum_document_lists_are_accepted_and_large_named_lists_fail_before_nodes() {
    let app = App::memory();
    app.mount(text_list().items((0..256).map(|_| text_list_item())))
        .unwrap();
    assert_eq!(app.tree().len(), 257);
    let app = App::memory();
    app.mount(data_list().items((0..256).map(|_| data_list_item())))
        .unwrap();
    assert_eq!(app.tree().len(), 1 + 256 * 4);
}

#[test]
fn document_item_explicit_theme_and_size_remain_effective_after_parent_inheritance() {
    let app = App::memory();
    app.mount(
        text_list().theme(sapid::Theme::green_dark()).item(
            text_list_item()
                .theme(sapid::Theme::blue_light())
                .size(sapid::Size::Small),
        ),
    )
    .unwrap();
    let tree = app.tree();
    let root = tree.roots()[0];
    let item = tree.node(root).unwrap().children[0];
    let descriptor = tree.node(item).unwrap().descriptor.view();
    assert_eq!(
        descriptor.style.color,
        Some(sapid::Theme::blue_light().foreground)
    );
    assert!(
        descriptor
            .classes
            .iter()
            .any(|class| class == "sapid-size-small")
    );
    drop(tree);
    let app = App::memory();
    app.mount(
        data_list().theme(sapid::Theme::green_dark()).item(
            data_list_item()
                .theme(sapid::Theme::blue_light())
                .term("Term"),
        ),
    )
    .unwrap();
    let tree = app.tree();
    let root = tree.roots()[0];
    let pair = tree.node(root).unwrap().children[0];
    let term = tree.node(pair).unwrap().children[0];
    assert_eq!(
        tree.node(term).unwrap().descriptor.view().style.color,
        Some(sapid::Theme::blue_light().foreground)
    );
}
