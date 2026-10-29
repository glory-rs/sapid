use glory::apace::shell::WindowApp;
use glory::render::{Query, QueryValue};
use glory::style::{Length, Style};
use std::{cell::RefCell, rc::Rc};
#[test]
fn pill_fits_content_in_a_normal_block_parent_for_every_theme() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.mount(
            glory::ui::view()
                .style(Style::new().width(Length::Px(400.0)))
                .fill(
                    sapid::pill()
                        .theme(theme)
                        .content("Rust")
                        .removable("Remove Rust"),
                ),
        )
        .unwrap();
        let app = host.app();
        app.settle(16).unwrap();
        let root = app.tree().roots()[0];
        let pill = app.tree().node(root).unwrap().children[0];
        let result = Rc::new(RefCell::new(None));
        let reply = result.clone();
        app.request(Query::Bounds(pill), move |value| {
            *reply.borrow_mut() = Some(value.unwrap())
        })
        .unwrap();
        app.pump().unwrap();
        let QueryValue::Bounds(rect) = result.borrow_mut().take().unwrap() else {
            panic!("bounds")
        };
        assert!(
            rect.width > 20.0 && rect.width < 200.0 && rect.height > 0.0,
            "{rect:?}"
        );
        app.clear().unwrap();
    }
}
