use glory::blitz::{Raster, shell::WindowApp};
use glory::prelude::*;
use glory::render::{Action, Event, NodeId, Query, QueryValue, Rect};
use std::{cell::RefCell, rc::Rc};
fn nodes(app: &App, class: &str) -> Vec<NodeId> {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    let mut out = vec![];
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if node.descriptor.view().classes.iter().any(|c| c == class) {
            out.push(id);
        }
        pending.extend(node.children.iter().copied());
    }
    out
}
fn bounds(app: &App, node: NodeId) -> Rect {
    let result = Rc::new(RefCell::new(None));
    let sink = result.clone();
    app.request(Query::Bounds(node), move |reply| {
        *sink.borrow_mut() = Some(reply.unwrap())
    })
    .unwrap();
    app.pump().unwrap();
    let QueryValue::Bounds(rect) = result.borrow_mut().take().unwrap() else {
        panic!()
    };
    rect
}
fn observe(host: &WindowApp) {
    for _ in 0..8 {
        host.observe(&std::task::Context::from_waker(std::task::Waker::noop()))
            .unwrap();
    }
}
#[test]
fn four_theme_real_native_window_geometry_retained_editors_and_actual_pixels() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let host = WindowApp::new();
        host.renderer().set_viewport(640, 420, 1.).unwrap();
        let app = host.app();
        let current = app.scope().cage(20usize);
        let editor = app.scope().cage(None);
        host.mount(
            sapid::carousel()
                .theme(theme)
                .count(1_000_000usize)
                .current(current)
                .overscan(3)
                .style(Style::new().width(Length::Px(500.)))
                .render_slide(move |index| {
                    let mut input = glory::ui::input()
                        .class(format!("native-carousel-editor-{index}"))
                        .value(format!("Draft {index}"));
                    if index == 21 {
                        input = input.node_ref(editor);
                    }
                    glory::ui::view()
                        .style(
                            Style::new()
                                .height(Length::Px(140.))
                                .padding(Edges::all(12.)),
                        )
                        .fill((
                            glory::ui::text().content(format!("Native slide {index}")),
                            input,
                        ))
                }),
        )
        .unwrap();
        observe(&host);
        assert_eq!(nodes(app, "sapid-carousel-slide").len(), 7);
        let root = nodes(app, "sapid-carousel")[0];
        let rect = bounds(app, root);
        assert!((rect.width - 500.).abs() < 1. && rect.height > 170.);
        let input = editor.get().unwrap();
        let physical = host
            .renderer()
            .engine_id(input, glory::html::Part::Main)
            .unwrap();
        current.revise(|value| *value = 21);
        app.flush().unwrap();
        observe(&host);
        assert_eq!(editor.get(), Some(input));
        assert_eq!(
            host.renderer()
                .engine_id(input, glory::html::Part::Main)
                .unwrap(),
            physical
        );
        let input_box = bounds(app, input);
        assert!(input_box.width > 100. && input_box.height > 10. && input_box.x >= rect.x);
        app.scope().action(Action::Focus(input)).unwrap();
        app.flush().unwrap();
        let result = Rc::new(RefCell::new(None));
        let reply = result.clone();
        app.request(Query::Focus(input), move |value| {
            *reply.borrow_mut() = Some(value.unwrap())
        })
        .unwrap();
        app.pump().unwrap();
        assert_eq!(*result.borrow(), Some(QueryValue::Focus(true)));
        let mut raster = Raster::with_limit(640 * 420 * 4);
        let frame = raster.draw(host.renderer()).unwrap();
        assert!(frame.rendered);
        assert!(
            raster
                .pixels()
                .as_chunks::<4>()
                .0
                .iter()
                .any(|pixel| pixel[3] > 0 && pixel[0] != pixel[1])
        );
        let previous = nodes(app, "sapid-carousel-previous")[0];
        app.dispatch(previous, Event::Activate).unwrap();
        assert_eq!(
            current.get(),
            21,
            "controlled source stays application-owned"
        );
        host.renderer().set_viewport(390, 420, 1.).unwrap();
        observe(&host);
        assert!(bounds(app, root).width <= 390.);
    }
}

#[test]
fn native_autoplay_tracks_hover_and_uses_actual_owner_executor() {
    use blitz_dom::Document;
    use blitz_traits::events::{
        BlitzPointerEvent, BlitzPointerId, MouseEventButton, MouseEventButtons, Point,
        PointerCoords, UiEvent,
    };
    use std::{
        cell::Cell,
        future::Future,
        pin::Pin,
        task::{Context, Poll, Waker},
    };
    struct Clock {
        ready: Rc<Cell<bool>>,
        wake: Rc<RefCell<Option<Waker>>>,
    }
    impl Future for Clock {
        type Output = ();
        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
            if self.ready.replace(false) {
                Poll::Ready(())
            } else {
                *self.wake.borrow_mut() = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
    let mut host = WindowApp::new();
    host.renderer().set_viewport(640, 420, 1.).unwrap();
    let app = host.app();
    let current = app.scope().cage(0usize);
    let paused = app.scope().cage(false);
    let ready = Rc::new(Cell::new(false));
    let wake = Rc::new(RefCell::new(None::<Waker>));
    let make_ready = ready.clone();
    let make_wake = wake.clone();
    host.mount(
        sapid::carousel()
            .count(3usize)
            .current(current)
            .looping(true)
            .paused(paused)
            .autoplay(move || {
                Box::pin(Clock {
                    ready: make_ready.clone(),
                    wake: make_wake.clone(),
                })
            })
            .on_current_change(move |next| current.revise(|v| *v = next))
            .render_slide(|n| glory::ui::text().content(n.to_string())),
    )
    .unwrap();
    assert!(wake.borrow().is_none(), "spawn never polls synchronously");
    observe(&host);
    assert!(wake.borrow().is_some());
    ready.set(true);
    wake.borrow().as_ref().unwrap().wake_by_ref();
    observe(&host);
    assert_eq!(current.get(), 1);
    let root = nodes(host.app(), "sapid-carousel")[0];
    let area = bounds(host.app(), root);
    let move_pointer = |host: &mut WindowApp, x: f32, y: f32| {
        host.handle_ui_event(UiEvent::PointerMove(BlitzPointerEvent {
            id: BlitzPointerId::Mouse,
            is_primary: true,
            coords: PointerCoords {
                page_x: x,
                page_y: y,
                client_x: x,
                client_y: y,
                screen_x: x,
                screen_y: y,
            },
            button: MouseEventButton::Main,
            buttons: MouseEventButtons::empty(),
            mods: Default::default(),
            details: Default::default(),
            element: Point::default(),
            active_pointers: Default::default(),
        }));
        observe(host);
    };
    let hover_wake = wake.borrow().as_ref().unwrap().clone();
    move_pointer(&mut host, (area.x + 5.) as f32, (area.y + 5.) as f32);
    ready.set(true);
    hover_wake.wake();
    observe(&host);
    assert_eq!(
        current.get(),
        1,
        "real native hover revokes the current wait"
    );
    ready.set(false);
    move_pointer(&mut host, 639., 419.);
    ready.set(true);
    wake.borrow().as_ref().unwrap().wake_by_ref();
    observe(&host);
    assert_eq!(current.get(), 2, "leaving the carousel resumes its clock");
    let late = wake.borrow().as_ref().unwrap().clone();
    paused.revise(|v| *v = true);
    host.app().flush().unwrap();
    ready.set(true);
    late.wake();
    observe(&host);
    assert_eq!(current.get(), 2, "retired native clock cannot publish");
}
