use glory::apace::{Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{Action, DragPhase, Event, NodeId, Query, QueryValue, Rect};
use sapid::{FloatingWindowHandle, WindowBounds, WindowCommand, floating_window};
use std::{cell::RefCell, rc::Rc};
fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut todo = tree.roots().to_vec();
    while let Some(id) = todo.pop() {
        let node = tree.node(id).unwrap();
        if node.descriptor.view().classes.iter().any(|c| c == class) {
            return id;
        }
        todo.extend(&node.children)
    }
    panic!("missing {class}")
}
fn bounds(app: &App, id: NodeId) -> Rect {
    let out = Rc::new(RefCell::new(None));
    let reply = out.clone();
    app.request(Query::Bounds(id), move |value| {
        *reply.borrow_mut() = Some(value.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let QueryValue::Bounds(rect) = out.borrow_mut().take().unwrap() else {
        panic!("bounds")
    };
    rect
}
fn drag(app: &App, target: NodeId, phase: DragPhase, x: f64, y: f64) {
    app.dispatch(
        target,
        Event::Drag {
            from: target,
            phase,
            x,
            y,
        },
    )
    .unwrap();
}
#[test]
fn four_theme_windows_move_resize_real_boxes_keep_native_editor_identity_and_paint_the_surface() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(900, 700, 1.).unwrap();
        let app = host.app();
        let editor = app.scope().cage(None);
        let expected = theme.surface;
        host.mount(
            floating_window("native-window")
                .theme(theme)
                .title("Native moving window")
                .content(move || {
                    sapid::text_field()
                        .name("Stable native draft")
                        .node_ref(editor)
                        .default_value("draft")
                }),
        )
        .unwrap();
        app.settle(16).unwrap();
        let panel = find(app, "sapid-floating-window");
        let caption = find(app, "sapid-window-caption");
        let grip = find(app, "sapid-window-resize-grip");
        let editor = editor.get().unwrap();
        let physical = host
            .renderer()
            .engine_id(editor, glory::html::Part::Main)
            .unwrap();
        assert_eq!(
            bounds(app, panel),
            Rect {
                x: 48.,
                y: 48.,
                width: 400.,
                height: 280.
            }
        );
        let origin = bounds(app, caption);
        drag(app, caption, DragPhase::Start, origin.x + 8., origin.y + 8.);
        drag(
            app,
            caption,
            DragPhase::Drop,
            origin.x + 158.,
            origin.y + 38.,
        );
        app.settle(16).unwrap();
        let moved = bounds(app, panel);
        assert_eq!((moved.x, moved.y), (198., 78.));
        let origin = bounds(app, grip);
        drag(app, grip, DragPhase::Start, origin.x + 8., origin.y + 8.);
        drag(app, grip, DragPhase::Drop, origin.x + 108., origin.y + 68.);
        app.settle(16).unwrap();
        let sized = bounds(app, panel);
        assert_eq!((sized.width, sized.height), (500., 340.));
        assert_eq!(
            host.renderer()
                .engine_id(editor, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        let mut raster = Raster::new();
        raster.draw(host.renderer()).unwrap();
        let frame = raster.frame().unwrap();
        let offset = ((sized.y as usize + 330) * frame.width as usize + sized.x as usize + 100) * 4;
        assert_eq!(
            &raster.pixels()[offset..offset + 4],
            &[expected.0, expected.1, expected.2, expected.3]
        );
        assert_eq!(app.stats().queries, 0);
        drag(app, caption, DragPhase::Start, sized.x + 10., sized.y + 10.);
        assert!(app.stats().queries <= 1);
        drag(app, grip, DragPhase::Start, sized.x + 490., sized.y + 330.);
        assert!(
            app.stats().queries <= 1,
            "move and resize share one current query"
        );
        drag(
            app,
            caption,
            DragPhase::Cancel,
            sized.x + 10.,
            sized.y + 10.,
        );
        drag(app, grip, DragPhase::Cancel, sized.x + 490., sized.y + 330.);
        assert_eq!(app.stats().queries, 0);
        drag(app, caption, DragPhase::Start, sized.x + 10., sized.y + 10.);
        app.clear().unwrap();
        app.pump().unwrap();
        assert_eq!(app.stats().queries, 0);
        assert_eq!(host.renderer().inspect(|doc| doc.top_layers().len()), 0);
    }
}
#[test]
fn native_controlled_refusal_disabled_fieldset_and_owner_cleanup_are_real() {
    let host = WindowApp::new();
    let app = host.app();
    let geometry = app.scope().cage(WindowBounds::default());
    let enabled = app.scope().cage(true);
    let editor = app.scope().cage(None);
    let handle = Rc::new(RefCell::new(None::<FloatingWindowHandle>));
    let output = handle.clone();
    let calls = app.scope().cage(0usize);
    host.mount(
        floating_window("native-held")
            .bounds(geometry)
            .enabled(enabled)
            .on_ready(move |value| *output.borrow_mut() = Some(value))
            .on_bounds_change(move |_| calls.revise(|value| *value += 1))
            .content(move || {
                sapid::text_field()
                    .name("Disabled native draft")
                    .node_ref(editor)
            }),
    )
    .unwrap();
    app.settle(16).unwrap();
    let handle = handle.borrow().clone().unwrap();
    let panel = find(app, "sapid-floating-window");
    handle.command(WindowCommand::MoveRight).unwrap();
    app.settle(16).unwrap();
    assert_eq!(bounds(app, panel).x, 48.);
    assert_eq!(calls.get(), 1);
    enabled.revise(|value| *value = false);
    app.settle(16).unwrap();
    assert!(
        app.scope()
            .action(Action::Focus(editor.get().unwrap()))
            .is_err()
    );
    handle.command(WindowCommand::MoveRight).unwrap();
    app.settle(16).unwrap();
    assert_eq!(calls.get(), 1);
    app.clear().unwrap();
    assert!(handle.raise().is_err());
    assert!(handle.set_bounds(WindowBounds::default()).is_err());
    assert_eq!(host.renderer().inspect(|doc| doc.top_layers().len()), 0);
}
