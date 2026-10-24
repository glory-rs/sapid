use glory::render::{Event, NodeId, Orientation, Props, Role, SemanticId, Semantics};
use glory::style::{Edges, FlexDirection, Length, Style};
use glory::ui::App;
use sapid::{Size, Theme, action_icon, burger, button, button_group, close_button};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

fn root(app: &App) -> NodeId {
    app.tree().roots()[0]
}
#[test]
fn icon_action_is_named_native_button_with_real_content_and_square_sizes() {
    for (size, side) in [
        (Size::Small, 30.0),
        (Size::Medium, 36.0),
        (Size::Large, 42.0),
    ] {
        let app = App::memory();
        app.mount(
            action_icon("Refresh")
                .size(size)
                .icon(glory::ui::text().content("↻")),
        )
        .unwrap();
        let d = app
            .tree()
            .node(root(&app))
            .unwrap()
            .descriptor
            .view()
            .to_owned();
        assert_eq!(d.name.as_deref(), Some("Refresh"));
        assert!(matches!(d.props, Props::Button(_)));
        assert_eq!(d.style.width, Some(Length::Px(side)));
        assert_eq!(d.style.height, Some(Length::Px(side)));
        assert_eq!(app.tree().len(), 2);
    }
}
#[test]
fn missing_names_or_icon_fail_before_creation() {
    let app = App::memory();
    assert!(
        app.mount(action_icon("").icon(glory::ui::text().content("!")))
            .is_err()
    );
    assert!(app.mount(action_icon("Action")).is_err());
    assert!(app.mount(close_button().name(" ")).is_err());
    assert!(app.mount(burger().labels("", "Close")).is_err());
    assert_eq!(app.tree().len(), 0);
}
#[test]
fn close_button_reuses_disabled_native_activation_and_observer_order() {
    let app = App::memory();
    let enabled = app.scope().cage(false);
    let order = Rc::new(RefCell::new(Vec::new()));
    let first = order.clone();
    let second = order.clone();
    app.mount(
        close_button()
            .name("Dismiss notification")
            .enabled(enabled)
            .on_press(move || first.borrow_mut().push(1))
            .on_press(move || second.borrow_mut().push(2)),
    )
    .unwrap();
    let id = root(&app);
    assert!(!app.dispatch(id, Event::Activate).unwrap());
    enabled.revise(|v| *v = true);
    app.flush().unwrap();
    assert!(app.dispatch(id, Event::Activate).unwrap());
    assert_eq!(*order.borrow(), [1, 2]);
    assert_eq!(root(&app), id);
}
#[test]
fn burger_local_expansion_updates_name_glyph_and_controls_without_new_identity() {
    let app = App::memory();
    let target = SemanticId::new("navigation").unwrap();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let seen = calls.clone();
    app.mount(
        burger()
            .controls(target.clone())
            .labels("Show menu", "Hide menu")
            .on_open_change(move |value| seen.borrow_mut().push(value)),
    )
    .unwrap();
    let id = root(&app);
    assert_eq!(
        app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref(),
        Some("Show menu")
    );
    assert!(app.dispatch(id, Event::Activate).unwrap());
    app.flush().unwrap();
    let d = app.tree().node(id).unwrap().descriptor.view().to_owned();
    assert_eq!(d.name.as_deref(), Some("Hide menu"));
    assert!(matches!(d.props,Props::Button(ref p) if p.label.is_empty()));
    assert_eq!(d.semantics.as_ref().unwrap().pressed, Some(true));
    let glyph = app.tree().node(id).unwrap().children[0];
    assert!(
        matches!(app.tree().node(glyph).unwrap().descriptor.view().props,Props::Graphic(p) if p.commands.len()==6 && p.alternative.is_none())
    );
    assert_eq!(d.semantics.as_ref().unwrap().expanded, Some(true));
    assert_eq!(d.semantics.as_ref().unwrap().controls, Some(target));
    assert_eq!(*calls.borrow(), [true]);
    assert_eq!(root(&app), id);
}
#[test]
fn burger_controlled_rejection_disabled_and_external_silent_updates() {
    let app = App::memory();
    let open = app.scope().cage(false);
    let enabled = app.scope().cage(false);
    let calls = Rc::new(Cell::new(0));
    let seen = calls.clone();
    app.mount(
        burger()
            .open(open)
            .default_open(true)
            .enabled(enabled)
            .on_open_change(move |_| seen.set(seen.get() + 1)),
    )
    .unwrap();
    let id = root(&app);
    assert!(!app.dispatch(id, Event::Activate).unwrap());
    enabled.revise(|v| *v = true);
    app.flush().unwrap();
    assert!(app.dispatch(id, Event::Activate).unwrap());
    app.flush().unwrap();
    assert!(!open.get());
    assert_eq!(calls.get(), 1);
    assert_eq!(
        app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .expanded,
        Some(false)
    );
    open.revise(|v| *v = true);
    app.flush().unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(
        app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .semantics
            .as_ref()
            .unwrap()
            .expanded,
        Some(true)
    );
}
#[test]
fn burger_internal_behavior_precedes_typed_callback_and_generic_observers() {
    let app = App::memory();
    let order = Rc::new(RefCell::new(Vec::new()));
    let first = order.clone();
    let second = order.clone();
    app.mount(
        burger()
            .on_open_change(move |open| first.borrow_mut().push(open))
            .on_press(move || second.borrow_mut().push(false)),
    )
    .unwrap();
    app.dispatch(root(&app), Event::Activate).unwrap();
    app.flush().unwrap();
    assert_eq!(*order.borrow(), [true, false]);
}
#[test]
fn all_themes_unstyled_and_explicit_overrides_are_portable() {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        let app = App::memory();
        app.mount(
            close_button()
                .theme(theme.clone())
                .style(Style::new().radius(0.0)),
        )
        .unwrap();
        let d = app
            .tree()
            .node(root(&app))
            .unwrap()
            .descriptor
            .view()
            .to_owned();
        assert_eq!(d.style.background, Some(theme.input));
        assert_eq!(d.style.radius, Some(0.0));
    }
    let app = App::memory();
    app.mount(burger().unstyled().style(Style::new().radius(1.0)))
        .unwrap();
    assert_eq!(
        app.tree()
            .node(root(&app))
            .unwrap()
            .descriptor
            .view()
            .style
            .clone(),
        Style::new().radius(1.0)
    );
}
#[test]
fn disclosure_relations_resolve_to_application_owned_panel() {
    let target = SemanticId::new("menu").unwrap();
    let html = glory::ssr::render(
        glory::ui::col().fill((
            burger().controls(target.clone()).default_open(true),
            glory::ui::view()
                .semantics(Some(Box::new(Semantics {
                    id: Some(target),
                    ..Default::default()
                })))
                .fill(glory::ui::text().content("Panel")),
        )),
    )
    .unwrap();
    assert!(html.contains("aria-expanded=\"true\""));
    assert!(html.contains("aria-controls=\"menu\""));
}
#[test]
fn ssr_names_and_icons_escape_and_keep_type_button() {
    let html = glory::ssr::render(glory::ui::row().fill((
        action_icon("Action <&>").icon(glory::ui::text().content("<&>")),
        close_button(),
        burger(),
    )))
    .unwrap();
    assert!(html.contains("aria-label=\"Action &lt;&amp;&gt;\""));
    assert!(html.contains("&lt;&amp;&gt;"));
    assert_eq!(html.matches("type=\"button\"").count(), 3);
    assert!(!html.contains("<script"));
}
#[test]
fn callbacks_and_references_retire_with_owner() {
    struct Probe(Rc<Cell<usize>>);
    impl Drop for Probe {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let app = App::memory();
    let reference = app.scope().cage(None);
    let dropped = Rc::new(Cell::new(0));
    let probe = Probe(dropped.clone());
    let baseline = app.runtime_stats();
    let mounted = app
        .scope()
        .mount_with(|| {
            burger().node_ref(reference).on_open_change(move |_| {
                let _ = &probe;
            })
        })
        .unwrap();
    app.flush().unwrap();
    let id = reference.get().unwrap();
    drop(mounted);
    app.flush().unwrap();
    assert_eq!(reference.get(), None);
    assert_eq!(dropped.get(), 1);
    assert_eq!(app.stats().handlers, 0);
    assert_eq!(app.runtime_stats().edges, baseline.edges);
    assert!(!app.dispatch(id, Event::Activate).unwrap());
}

#[test]
fn button_group_has_single_outer_border_ordered_buttons_and_directional_separators() {
    for orientation in [Orientation::Horizontal, Orientation::Vertical] {
        let app = App::memory();
        app.mount(
            button_group()
                .name("Document actions")
                .orientation(orientation)
                .buttons([
                    button().label("Save"),
                    button().label("Discard"),
                    button().label("More"),
                ]),
        )
        .unwrap();
        let group = root(&app);
        let d = app.tree().node(group).unwrap().descriptor.view().to_owned();
        assert_eq!(d.role, Some(Role::Group));
        assert_eq!(d.name.as_deref(), Some("Document actions"));
        assert_eq!(d.style.border_width, Some(Edges::all(1.0)));
        assert_eq!(
            d.style.flex_direction,
            Some(if orientation == Orientation::Horizontal {
                FlexDirection::Row
            } else {
                FlexDirection::Column
            })
        );
        let children = app.tree().node(group).unwrap().children.clone();
        for (index, id) in children.iter().enumerate() {
            let d = app.tree().node(*id).unwrap().descriptor.view().to_owned();
            assert!(
                matches!(d.props,Props::Button(ref p) if p.label==["Save","Discard","More"][index])
            );
            assert_eq!(d.style.radius, Some(0.0));
            let separator = if index == 2 { 0.0 } else { 1.0 };
            assert_eq!(
                d.style.border_width,
                Some(if orientation == Orientation::Horizontal {
                    Edges {
                        top: 0.0,
                        bottom: 0.0,
                        left: 0.0,
                        right: separator,
                    }
                } else {
                    Edges {
                        top: 0.0,
                        bottom: separator,
                        left: 0.0,
                        right: 0.0,
                    }
                })
            );
        }
    }
}

#[test]
fn button_group_disabled_combines_with_child_state_and_does_not_rebuild() {
    let app = App::memory();
    let enabled = app.scope().cage(true);
    let child_enabled = app.scope().cage(false);
    let calls = Rc::new(Cell::new(0));
    let first = calls.clone();
    let second = calls.clone();
    app.mount(
        button_group()
            .enabled(enabled)
            .button(
                button()
                    .label("A")
                    .on_press(move || first.set(first.get() + 1)),
            )
            .button(
                button()
                    .label("B")
                    .enabled(child_enabled)
                    .on_press(move || second.set(second.get() + 1)),
            ),
    )
    .unwrap();
    let children = app.tree().node(root(&app)).unwrap().children.clone();
    assert!(app.dispatch(children[0], Event::Activate).unwrap());
    assert!(!app.dispatch(children[1], Event::Activate).unwrap());
    enabled.revise(|v| *v = false);
    child_enabled.revise(|v| *v = true);
    app.flush().unwrap();
    for id in &children {
        assert!(!app.dispatch(*id, Event::Activate).unwrap());
    }
    enabled.revise(|v| *v = true);
    app.flush().unwrap();
    assert!(app.dispatch(children[1], Event::Activate).unwrap());
    assert_eq!(calls.get(), 2);
    assert_eq!(app.tree().node(root(&app)).unwrap().children, children);
}

#[test]
fn button_group_preserves_dynamic_explicit_child_style_over_layout_and_shared_theme() {
    let app = App::memory();
    let style = app.scope().cage(Style::new().radius(7.0));
    app.mount(
        button_group()
            .theme(Theme::green_dark())
            .size(Size::Small)
            .button(button().label("A").style(style))
            .button(button().label("B")),
    )
    .unwrap();
    let children = app.tree().node(root(&app)).unwrap().children.clone();
    let first = children[0];
    let d = app.tree().node(first).unwrap().descriptor.view().to_owned();
    assert_eq!(d.style.radius, Some(7.0));
    assert_eq!(d.style.min_height, Some(Length::Px(30.0)));
    assert_eq!(d.style.background, Some(Theme::green_dark().accent));
    style.revise(|v| *v = Style::new().padding(Edges::all(2.0)));
    app.flush().unwrap();
    let d = app.tree().node(first).unwrap().descriptor.view().to_owned();
    assert_eq!(d.style.radius, Some(0.0));
    assert_eq!(d.style.padding, Some(Edges::all(2.0)));
    assert_eq!(
        d.style.border_width,
        Some(Edges {
            top: 0.0,
            bottom: 0.0,
            left: 0.0,
            right: 1.0
        })
    );
}

#[test]
fn unstyled_group_removes_owned_connection_and_button_theme_and_empty_groups_fail() {
    let app = App::memory();
    assert!(app.mount(button_group()).is_err());
    assert_eq!(app.tree().len(), 0);
    app.mount(
        button_group()
            .unstyled()
            .button(button().label("A").style(Style::new().radius(9.0))),
    )
    .unwrap();
    let id = root(&app);
    assert_eq!(
        app.tree().node(id).unwrap().descriptor.view().style.clone(),
        Style::new()
            .display(glory::style::Display::Flex)
            .flex_direction(FlexDirection::Row)
            .gap(0.0)
    );
    let child = app.tree().node(id).unwrap().children[0];
    assert_eq!(
        app.tree()
            .node(child)
            .unwrap()
            .descriptor
            .view()
            .style
            .clone(),
        Style::new().radius(9.0)
    );
    let app = App::memory();
    app.mount(button_group().joined(false).button(button().label("A")))
        .unwrap();
    assert_eq!(
        app.tree()
            .node(root(&app))
            .unwrap()
            .descriptor
            .view()
            .style
            .border_width,
        None
    );
}
