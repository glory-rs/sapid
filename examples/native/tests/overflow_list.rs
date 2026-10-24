use glory::blitz::shell::WindowApp;
use glory::prelude::*;
use glory::render::{NodeId, Query, QueryValue, Rect};
use std::{cell::RefCell, rc::Rc};
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
fn explicit_inline_mode_has_real_four_theme_intrinsic_widths_without_claiming_native_popovers()
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
        let default = WindowApp::new();
        assert!(
            default
                .mount(sapid::overflow_list("popup").item(sapid::overflow_item("a")))
                .is_err()
        );
    }
}
