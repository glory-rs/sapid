#![cfg(feature = "chat")]
use glory::render::{Event, FocusMode, Navigation, NodeId, Props, Role};
use glory::ui::App;
use sapid::{suggestion, suggestion_group};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
fn find(app: &App, predicate: impl Fn(glory::render::DescriptorView<'_>) -> bool) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if predicate(node.descriptor.view()) {
            return id;
        }
        pending.extend(node.children.iter().copied());
    }
    panic!("missing node")
}
fn button(app: &App, label: &str) -> NodeId {
    find(
        app,
        |node| matches!(node.props,Props::Button(value)if value.label==label),
    )
}
#[test]
fn suggestion_chooses_explicit_payload_and_source_before_group_callback_without_sending() {
    let app = App::memory();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let single = calls.clone();
    let group = calls.clone();
    app.mount(
        suggestion_group("actions")
            .suggestion(
                suggestion("explain")
                    .label("Explain")
                    .value("Explicit application text\nSecond line")
                    .source("Public notes")
                    .on_choose(move |choice| {
                        assert_eq!(choice.key, "explain");
                        assert_eq!(choice.value, "Explicit application text\nSecond line");
                        assert_eq!(choice.source.as_deref(), Some("Public notes"));
                        single.borrow_mut().push("item");
                    }),
            )
            .on_choose(move |_| group.borrow_mut().push("group")),
    )
    .unwrap();
    find(&app, |node| node.role == Some(Role::Toolbar));
    app.dispatch(button(&app, "Explain · Public notes"), Event::Activate)
        .unwrap();
    assert_eq!(*calls.borrow(), ["item", "group"]);
    assert_eq!(app.runtime_stats().tasks, 0);
    app.clear().unwrap();
    assert_eq!(Rc::strong_count(&calls), 1);
    assert_eq!(app.stats().nodes, 0);
}
#[test]
fn real_toolbar_roving_skips_disabled_actions_and_preserves_one_tab_entry() {
    let app = App::memory();
    let enabled = app.scope().cage(true);
    app.mount(
        suggestion_group("navigation")
            .enabled(enabled)
            .suggestions([
                suggestion("first").label("First"),
                suggestion("disabled").label("Disabled").enabled(false),
                suggestion("last").label("Last"),
            ]),
    )
    .unwrap();
    let first = button(&app, "First");
    let last = button(&app, "Last");
    let disabled = button(&app, "Disabled");
    find(&app, |node| node.role == Some(Role::Toolbar));
    assert_eq!(
        app.tree().node(first).unwrap().descriptor.view().focus,
        FocusMode::TabStop
    );
    app.dispatch(
        first,
        Event::Navigate {
            from: first,
            command: Navigation::Next,
        },
    )
    .unwrap();
    assert_eq!(
        app.tree().node(last).unwrap().descriptor.view().focus,
        FocusMode::TabStop
    );
    assert_ne!(
        app.tree().node(first).unwrap().descriptor.view().focus,
        FocusMode::TabStop
    );
    assert!(!app.dispatch(disabled, Event::Activate).unwrap());
    app.dispatch(
        last,
        Event::Navigate {
            from: last,
            command: Navigation::First,
        },
    )
    .unwrap();
    assert_eq!(
        app.tree().node(first).unwrap().descriptor.view().focus,
        FocusMode::TabStop
    );
    enabled.revise(|value| *value = false);
    app.flush().unwrap();
    assert!(!app.tree().enabled(first));
    assert!(!app.dispatch(first, Event::Activate).unwrap());
}
#[test]
fn bounded_intake_duplicate_keys_and_payload_limits_reject_before_rendering() {
    let app = App::memory();
    let consumed = Rc::new(Cell::new(0));
    let output = consumed.clone();
    let config = suggestion_group("million").suggestions(
        std::iter::repeat_with(move || {
            output.set(output.get() + 1);
            suggestion("item")
        })
        .take(1_000_000),
    );
    assert_eq!(consumed.get(), 129);
    assert!(app.mount(config).is_err());
    assert!(app.tree().roots().is_empty());
    assert!(
        app.mount(
            suggestion_group("duplicates")
                .suggestion(suggestion("same"))
                .suggestion(suggestion("same"))
        )
        .is_err()
    );
    assert!(
        app.mount(suggestion("large").value("x".repeat(1_000_000)))
            .is_err()
    );
    assert!(app.tree().roots().is_empty());
}

#[test]
fn group_theme_reaches_item_styles_and_nested_css_classes() {
    let mut colors = sapid::BLUE.base;
    colors.accent = glory::style::Color(81, 36, 130, 255);
    let theme =
        sapid::Theme::from_palette("custom-night", sapid::ColorScheme::Dark, colors).unwrap();
    let app = App::memory();
    app.mount(
        suggestion_group("themed")
            .theme(theme.clone())
            .suggestion(suggestion("first").label("Themed action")),
    )
    .unwrap();
    let id = button(&app, "Themed action");
    let tree = app.tree();
    let descriptor = tree.node(id).unwrap().descriptor.view();
    assert_eq!(descriptor.style.background, Some(colors.accent));
    assert!(descriptor.classes.iter().any(|c| c == &theme.class()));
    assert!(
        !descriptor
            .classes
            .iter()
            .any(|c| c == "sapid-theme-blue-light")
    );
}
