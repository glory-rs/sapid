use glory::blitz::shell::WindowApp;
use glory::prelude::*;
use glory::render::{
    Action, NodeId, Props, Query, QueryValue, Rect, Role, SelectionDirection, TextSelection,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
fn nodes(app: &App, class: &str) -> Vec<NodeId> {
    let tree = app.tree();
    let mut todo = tree.roots().to_vec();
    let mut values = Vec::new();
    while let Some(id) = todo.pop() {
        let node = tree.node(id).unwrap();
        if node
            .descriptor
            .view()
            .classes
            .iter()
            .any(|value| value == class)
        {
            values.push(id)
        }
        todo.extend(node.children.iter().rev());
    }
    values
}
fn bounds(app: &App, id: NodeId) -> Rect {
    let output = Rc::new(RefCell::new(None));
    let reply = output.clone();
    app.request(Query::Bounds(id), move |result| {
        *reply.borrow_mut() = Some(result.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let QueryValue::Bounds(rect) = output.borrow_mut().take().unwrap() else {
        panic!("bounds")
    };
    rect
}
#[test]
fn explicit_inline_mode_keeps_real_four_theme_intrinsic_widths_and_editor_identity() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(640, 480, 1.).unwrap();
        let app = host.app();
        let title = app.scope().cage("Short".to_owned());
        let width = app.scope().cage(480.);
        let editor = app.scope().cage(None);
        let button_theme = theme.clone();
        host.mount(
            sapid::overflow_list("native")
                .overflow(false)
                .theme(theme.clone())
                .style(width.map(|width| Style::new().width(Length::Px(*width))))
                .item(
                    sapid::overflow_item("label").content(move || {
                        sapid::button().theme(button_theme.clone()).label(title)
                    }),
                )
                .item(sapid::overflow_item("editor").width(140.).content(move || {
                    glory::ui::input()
                        .node_ref(editor)
                        .value("Native draft")
                        .style(Style::new().width(Length::Percent(100.)))
                }))
                .item(
                    sapid::overflow_item("tail")
                        .content(|| glory::ui::text().content("Last item")),
                ),
        )
        .unwrap();
        app.settle(16).unwrap();
        let rows = nodes(app, "sapid-overflow-item");
        let initial = bounds(app, rows[0]);
        let input = editor.get().unwrap();
        let physical = host
            .renderer()
            .engine_id(input, glory::html::Part::Main)
            .unwrap();
        assert!(initial.width > 20. && initial.height > 20.);
        assert!((bounds(app, rows[1]).width - 140.).abs() < 0.5);
        title.revise(|value| *value = "A much longer native label".to_owned());
        app.flush().unwrap();
        app.settle(16).unwrap();
        assert!(bounds(app, rows[0]).width > initial.width + 80.);
        assert!(bounds(app, rows[1]).x > bounds(app, rows[0]).x + bounds(app, rows[0]).width);
        width.revise(|value| *value = 560.);
        app.flush().unwrap();
        assert!((bounds(app, nodes(app, "sapid-overflow-line")[0]).width - 560.).abs() < 0.5);
        assert_eq!(
            host.renderer()
                .engine_id(input, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        assert!(nodes(app, "sapid-layer").is_empty());
        app.clear().unwrap();
        assert_eq!(app.stats().queries, 0);
    }
}

fn observe(host: &WindowApp) {
    for _ in 0..32 {
        host.observe(&std::task::Context::from_waker(std::task::Waker::noop()))
            .unwrap();
    }
}
fn query(app: &App, query: Query) -> QueryValue {
    let output = Rc::new(RefCell::new(None));
    let reply = output.clone();
    app.request(query, move |result| {
        *reply.borrow_mut() = Some(result.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    output.borrow_mut().take().unwrap()
}
#[test]
fn native_popup_mode_measures_opens_real_anchor_layer_and_keeps_editor_focus_and_owner_in_four_themes()
 {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(640, 480, 1.).unwrap();
        let app = host.app();
        let width = app.scope().cage(300.);
        let editor = app.scope().cage(None);
        let handle = app.scope().cage(None);
        let value = app.scope().cage("Retained popup draft".to_owned());
        let built = Rc::new(Cell::new(0));
        let cleaned = Rc::new(Cell::new(0));
        let first = built.clone();
        let second = built.clone();
        let third = built.clone();
        let cleanup = cleaned.clone();
        let palette = theme.clone();
        host.mount(
            sapid::overflow_list("native-popup")
                .theme(theme.clone())
                .style(width.map(|width| Style::new().width(Length::Px(*width))))
                .initial_visible(1)
                .handle_ref(handle)
                .item(sapid::overflow_item("first").width(80.).content(move || {
                    first.set(first.get() + 1);
                    sapid::button().theme(palette.clone()).label("First item")
                }))
                .item(sapid::overflow_item("draft").width(140.).content(move || {
                    second.set(second.get() + 1);
                    let cleanup = cleanup.clone();
                    glory::ui::component(move |scope| {
                        let cleanup = cleanup.clone();
                        scope.on_cleanup(move || cleanup.set(cleanup.get() + 1))?;
                        input()
                            .name("Retained overflow editor")
                            .value(value)
                            .node_ref(editor)
                            .style(Style::new().width(Length::Percent(100.)))
                            .show_in(scope)
                    })
                }))
                .item(sapid::overflow_item("last").width(80.).content(move || {
                    third.set(third.get() + 1);
                    button().label("Last item")
                })),
        )
        .unwrap();
        observe(&host);
        let handle = handle.get().unwrap();
        assert_eq!(built.get(), 3);
        assert_eq!(handle.pending_queries().get(), 0);
        assert!(handle.visible_count().get() <= 1);
        assert!(handle.overflow_count().get() >= 2);
        let editor = editor.get().unwrap();
        let physical = host
            .renderer()
            .engine_id(editor, glory::html::Part::Main)
            .unwrap();
        let trigger = nodes(app, "sapid-popover-trigger")[0];
        let panel = nodes(app, "sapid-popover")[0];
        assert_eq!(
            app.tree().node(panel).unwrap().descriptor.view().role,
            Some(Role::Dialog)
        );
        assert_eq!(
            host.renderer()
                .inspect(|document| document.top_layers().len()),
            0
        );
        app.scope().action(Action::Focus(trigger)).unwrap();
        observe(&host);
        assert_eq!(query(app, Query::Focus(trigger)), QueryValue::Focus(true));
        app.dispatch(trigger, Event::Activate).unwrap();
        observe(&host);
        assert_eq!(
            host.renderer()
                .inspect(|document| document.top_layers().len()),
            1
        );
        let anchor = bounds(app, trigger);
        let popup = bounds(app, panel);
        assert!(
            popup.width > 140.
                && popup.height > 50.
                && popup.x >= 0.
                && popup.x + popup.width <= 640.1
                && popup.y >= anchor.y + anchor.height - 1.
                && popup.y + popup.height <= 480.1,
            "anchor {anchor:?}; popup {popup:?}"
        );
        let editor_rect = bounds(app, editor);
        let hit = host.renderer().inspect(|document| {
            document
                .hit(
                    (editor_rect.x + editor_rect.width / 2.) as f32,
                    (editor_rect.y + editor_rect.height / 2.) as f32,
                )
                .unwrap()
                .node_id
        });
        assert_eq!(hit, physical);
        app.scope().action(Action::Focus(editor)).unwrap();
        let revision = match app.tree().node(editor).unwrap().descriptor.view().props {
            Props::Input(value) => value.revision,
            _ => panic!(),
        };
        app.scope()
            .action(Action::SelectText {
                node: editor,
                revision,
                selection: TextSelection {
                    start: 2,
                    end: 8,
                    direction: SelectionDirection::Backward,
                },
            })
            .unwrap();
        observe(&host);
        assert_eq!(
            query(app, Query::Selection(editor)),
            QueryValue::Selection { start: 2, end: 8 }
        );
        let mut raster = glory::blitz::Raster::new();
        raster.draw(host.renderer()).unwrap();
        let color = theme.surface;
        assert!(
            raster
                .pixels()
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|pixel| **pixel == [color.0, color.1, color.2, color.3])
                .count()
                > 500
        );
        handle.set_open(false).unwrap();
        observe(&host);
        assert_eq!(
            host.renderer()
                .inspect(|document| document.top_layers().len()),
            0
        );
        assert_eq!(query(app, Query::Focus(trigger)), QueryValue::Focus(true));
        handle.set_open(true).unwrap();
        observe(&host);
        app.scope().action(Action::Focus(editor)).unwrap();
        app.scope()
            .action(Action::SelectText {
                node: editor,
                revision,
                selection: TextSelection {
                    start: 2,
                    end: 8,
                    direction: SelectionDirection::Backward,
                },
            })
            .unwrap();
        observe(&host);
        width.revise(|width| *width = 560.);
        handle.refresh().unwrap();
        observe(&host);
        assert_eq!(handle.visible_count().get(), 3);
        assert_eq!(handle.overflow_count().get(), 0);
        assert_eq!(built.get(), 3);
        assert_eq!(cleaned.get(), 0);
        assert_eq!(
            host.renderer()
                .engine_id(editor, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        assert_eq!(query(app, Query::Focus(editor)), QueryValue::Focus(true));
        assert_eq!(
            query(app, Query::Selection(editor)),
            QueryValue::Selection { start: 2, end: 8 }
        );
        assert_eq!(value.get(), "Retained popup draft");
        app.clear().unwrap();
        assert_eq!(cleaned.get(), 1);
        assert!(handle.refresh().is_err());
        assert_eq!(app.stats().queries, 0);
        assert_eq!(app.stats().nodes, 0);
        assert_eq!(host.renderer().stats().nodes, 0);
    }
}
