#[path = "../../web/src/document_form.rs"]
mod document_form;
use glory::apace::shell::WindowApp;
use glory::render::{NodeId, Query, QueryValue};
use glory::style::{Length, Style};
use glory::ui::App;
use std::{cell::RefCell, rc::Rc};
fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let n = tree.node(id).unwrap();
        if n.descriptor.view().classes.iter().any(|c| c == class) {
            return id;
        }
        pending.extend(n.children.iter().copied());
    }
    panic!("missing {class}")
}
#[test]
fn document_widgets_have_real_native_layout_for_every_theme_and_retire() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.mount(
            glory::ui::col()
                .style(Style::new().width(Length::Px(480.0)))
                .fill(document_form::document_form(theme)),
        )
        .unwrap();
        let app = host.app();
        app.settle(16).unwrap();
        for class in [
            "sapid-code",
            "sapid-kbd",
            "sapid-mark",
            "sapid-text-list",
            "sapid-blockquote",
            "sapid-data-list",
            "sapid-data-list-term",
            "sapid-data-list-definition",
            "sapid-item",
        ] {
            let result = Rc::new(RefCell::new(None));
            let reply = result.clone();
            app.request(Query::Bounds(find(app, class)), move |value| {
                *reply.borrow_mut() = Some(value.unwrap())
            })
            .unwrap();
            app.pump().unwrap();
            let QueryValue::Bounds(rect) = result.borrow_mut().take().unwrap() else {
                panic!("bounds expected")
            };
            assert!(rect.width > 0.0 && rect.height > 0.0, "{class}: {rect:?}");
        }
        app.clear().unwrap();
        assert!(app.tree().is_empty());
        assert_eq!(app.stats().handlers, 0);
    }
}
