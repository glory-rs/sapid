use glory::blitz::shell::WindowApp;
use glory::prelude::*;
use glory::render::{NodeId, Query, QueryValue, Rect};
use std::{cell::RefCell, rc::Rc};
fn bounds(app: &App, id: NodeId) -> Rect {
    let result = Rc::new(RefCell::new(None));
    let reply = result.clone();
    app.request(Query::Bounds(id), move |value| {
        *reply.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let QueryValue::Bounds(rect) = result.borrow_mut().take().unwrap() else {
        panic!("bounds")
    };
    rect
}
#[test]
fn native_unstyled_range_family_keeps_real_space_and_does_not_overlap_following_editor() {
    for (size, cross) in [
        (sapid::Size::ExtraSmall, 24.),
        (sapid::Size::Small, 30.),
        (sapid::Size::Medium, 36.),
        (sapid::Size::Large, 42.),
        (sapid::Size::ExtraLarge, 50.),
    ] {
        let host = WindowApp::new();
        let app = host.app();
        let a = app.scope().cage(None);
        let b = app.scope().cage(None);
        let c = app.scope().cage(None);
        let d = app.scope().cage(None);
        host.mount(
            sapid::stack().gap(12.).fill((
                sapid::slider("single").unstyled().size(size).node_ref(a),
                sapid::range_slider("pair")
                    .unstyled()
                    .size(size)
                    .node_ref(b),
                sapid::angle_slider("angle")
                    .unstyled()
                    .size(size)
                    .node_ref(c),
                glory::ui::input().name("Following editor").node_ref(d),
            )),
        )
        .unwrap();
        app.settle(16).unwrap();
        let boxes = [a, b, c, d].map(|r| bounds(app, r.get().unwrap()));
        for rect in &boxes[..2] {
            assert!((rect.width - 240.).abs() < 0.1);
            assert!((rect.height - cross).abs() < 0.1);
        }
        assert!((boxes[2].width - (cross * 3. + 12.)).abs() < 0.1);
        assert!((boxes[2].height - boxes[2].width).abs() < 0.1);
        for adjacent in boxes.windows(2) {
            assert!(
                adjacent[1].y >= adjacent[0].y + adjacent[0].height + 11.9,
                "overlap: {boxes:?}"
            );
        }
        app.clear().unwrap();
    }
}
