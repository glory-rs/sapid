#![cfg(feature = "chat")]
use glory::render::{Event, NodeId, Rect, SemanticId, VirtualPart};
use glory::ui::{App, list::ListStats};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[path = "../examples/web/src/message_scroller_form.rs"]
mod fixture;

#[test]
fn chat_fixture_constructs_only_the_live_window_in_four_themes() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let app = App::memory();
        app.mount(fixture::message_scroller_form(theme)).unwrap();
        assert!(app.stats().nodes < 1500, "{}", app.stats().nodes);
        app.clear().unwrap();
        assert_eq!(app.stats().nodes, 0);
    }
}
fn id(index: usize) -> SemanticId {
    SemanticId::new(format!("message-{index}")).unwrap()
}
fn index(key: &SemanticId) -> Option<usize> {
    key.as_str().strip_prefix("message-")?.parse().ok()
}
fn root(app: &App) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if node.descriptor.view().options.get::<VirtualPart>() == Some(&VirtualPart::Root) {
            return id;
        }
        pending.extend(node.children.iter().copied());
    }
    panic!("root")
}
fn scroll(y: f64) -> Event {
    Event::Scroll {
        x: 0.,
        y,
        ack: None,
        viewport: Rect {
            x: 0.,
            y: 0.,
            width: 500.,
            height: 200.,
        },
    }
}
#[test]
fn huge_history_reads_only_two_bounded_windows_and_keeps_no_retired_controller() {
    let app = App::memory();
    let reads = Rc::new(Cell::new(0));
    let calls = reads.clone();
    let handle = Rc::new(RefCell::new(None));
    let ready = handle.clone();
    let stats = Rc::new(RefCell::new(None::<ListStats>));
    let output = stats.clone();
    app.mount(
        sapid::message_scroller(
            100_000_000usize,
            move |at| {
                calls.set(calls.get() + 1);
                id(at)
            },
            index,
            |_, key| glory::ui::text().content(key.as_str().to_owned()),
        )
        .height(200.)
        .estimate(40.)
        .on_ready(move |value| *ready.borrow_mut() = Some(value))
        .inspect(move |value| *output.borrow_mut() = Some(value)),
    )
    .unwrap();
    let latest = stats.borrow().clone().unwrap();
    assert!(latest.rows <= 64);
    assert!(latest.range.start > 99_999_000);
    assert!(reads.get() < 40, "{}", reads.get());
    assert!(latest.height_slots <= 256);
    let handle = handle.borrow().clone().unwrap();
    assert!(handle.status().unwrap().following);
    app.dispatch(root(&app), scroll(400.)).unwrap();
    app.flush().unwrap();
    assert!(!handle.status().unwrap().following);
    handle.appended(2).unwrap();
    handle.appended(3).unwrap();
    assert_eq!(handle.status().unwrap().unread, 5);
    handle.follow_tail().unwrap();
    app.flush().unwrap();
    assert_eq!(handle.status().unwrap().unread, 0);
    assert!(stats.borrow().as_ref().unwrap().range.start > 99_999_000);
    app.clear().unwrap();
    assert!(handle.status().is_err());
    assert!(handle.follow_tail().is_err());
    assert!(handle.appended(1).is_err());
    assert!(handle.seek(0.).is_err());
}
#[test]
fn program_seek_is_bounded_pauses_following_and_can_return_to_tail() {
    let app = App::memory();
    let handle = Rc::new(RefCell::new(None));
    let ready = handle.clone();
    let stats = Rc::new(RefCell::new(None::<ListStats>));
    let output = stats.clone();
    app.mount(
        sapid::message_scroller(1000usize, id, index, |_, key| {
            glory::ui::text().content(key.as_str().to_owned())
        })
        .height(200.)
        .estimate(40.)
        .on_ready(move |value| *ready.borrow_mut() = Some(value))
        .inspect(move |value| *output.borrow_mut() = Some(value)),
    )
    .unwrap();
    let handle = handle.borrow().clone().unwrap();
    let before = stats.borrow().as_ref().unwrap().logical;
    for invalid in [
        f64::NAN,
        f64::INFINITY,
        -1.,
        glory::ui::list::MAX_EXTENT + 1.,
    ] {
        assert!(handle.seek(invalid).is_err());
        assert!(handle.status().unwrap().following);
    }
    app.flush().unwrap();
    assert_eq!(stats.borrow().as_ref().unwrap().logical, before);
    handle.seek(0.).unwrap();
    app.flush().unwrap();
    assert!(!handle.status().unwrap().following);
    assert_eq!(stats.borrow().as_ref().unwrap().logical, 0.);
    assert_eq!(stats.borrow().as_ref().unwrap().range.start, 0);
    handle.appended(3).unwrap();
    handle.seek(glory::ui::list::MAX_EXTENT).unwrap();
    app.flush().unwrap();
    assert_eq!(handle.status().unwrap().unread, 3);
    assert_eq!(stats.borrow().as_ref().unwrap().logical, before);
    assert!(!handle.status().unwrap().following);
    handle.follow_tail().unwrap();
    app.flush().unwrap();
    assert!(handle.status().unwrap().following);
    assert_eq!(handle.status().unwrap().unread, 0);
}
#[test]
fn prepend_preserves_reading_anchor_and_append_notifications_do_not_force_scroll() {
    let app = App::memory();
    let first = app.scope().cage(100usize);
    let count = app.scope().cage(1000usize);
    let revision = app.scope().cage(0u64);
    let stats = Rc::new(RefCell::new(None::<ListStats>));
    let output = stats.clone();
    let handle = Rc::new(RefCell::new(None));
    let ready = handle.clone();
    app.mount(
        sapid::message_scroller(
            count,
            move |at| id(first.get() + at),
            move |key| {
                index(key)
                    .and_then(|key| key.checked_sub(first.get()))
                    .filter(|at| *at < count.get())
            },
            |_, key| glory::ui::text().content(key.as_str().to_owned()),
        )
        .revision(revision)
        .height(200.)
        .estimate(40.)
        .initially_following(false)
        .on_ready(move |value| *ready.borrow_mut() = Some(value))
        .inspect(move |value| *output.borrow_mut() = Some(value)),
    )
    .unwrap();
    app.dispatch(root(&app), scroll(800.)).unwrap();
    app.flush().unwrap();
    let before = stats.borrow().as_ref().unwrap().logical;
    app.scope()
        .batch(|| {
            first.revise(|value| *value -= 10);
            count.revise(|value| *value += 10);
            revision.revise(|value| *value += 1);
        })
        .unwrap();
    app.flush().unwrap();
    assert_eq!(stats.borrow().as_ref().unwrap().logical, before + 400.);
    let handle = handle.borrow().clone().unwrap();
    assert_eq!(handle.status().unwrap().unread, 0);
    count.revise(|value| *value += 4);
    revision.revise(|value| *value += 1);
    handle.appended(4).unwrap();
    app.flush().unwrap();
    assert_eq!(stats.borrow().as_ref().unwrap().logical, before + 400.);
    assert_eq!(handle.status().unwrap().unread, 4);
    handle.appended(u64::MAX).unwrap_err();
    assert_eq!(handle.status().unwrap().unread, 4);
}
#[test]
fn invalid_window_configuration_rejects_before_any_key_lookup() {
    let app = App::memory();
    let reads = Rc::new(Cell::new(0));
    let observed = reads.clone();
    assert!(
        app.mount(
            sapid::message_scroller(
                1_000_000usize,
                move |at| {
                    observed.set(observed.get() + 1);
                    id(at)
                },
                index,
                |_, _| glory::ui::text()
            )
            .row_limit(257)
        )
        .is_err()
    );
    assert_eq!(reads.get(), 0);
    assert!(app.tree().roots().is_empty());
}

#[test]
fn chat_prompt_and_named_window_are_both_retained_in_the_actual_tree() {
    let app = App::memory();
    app.mount(
        sapid::chat(
            sapid::message_scroller(1000usize, id, index, |_, key| {
                glory::ui::text().content(key.as_str().to_owned())
            })
            .height(200.)
            .name("Application history"),
        )
        .prompt(
            sapid::textarea()
                .name("Conversation draft")
                .default_value("Draft stays outside the window"),
        ),
    )
    .unwrap();
    let viewport = root(&app);
    assert_eq!(
        app.tree()
            .node(viewport)
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref(),
        Some("Application history")
    );
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    let mut input = false;
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        input |= matches!(node.descriptor.view().props,glory::render::Props::Input(input) if input.value=="Draft stays outside the window");
        pending.extend(node.children.iter().copied());
    }
    assert!(input);
}
