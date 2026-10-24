use glory::prelude::*;
use glory::render::{Event, EventKind, NodeId, Props};
use sapid::{Size, button};
use std::{cell::RefCell, rc::Rc};
fn root(app: &App) -> NodeId {
    app.tree().roots()[0]
}
fn label(app: &App) -> NodeId {
    let tree = app.tree();
    let mut ids = tree.roots().to_vec();
    while let Some(id) = ids.pop() {
        let n = tree.node(id).unwrap();
        if n.descriptor
            .view()
            .classes
            .iter()
            .any(|c| c == "sapid-button-label")
        {
            return id;
        }
        ids.extend(n.children.iter().copied());
    }
    panic!("label")
}
#[test]
fn five_sizes_share_control_tokens_and_icon_dimensions() {
    for (size, height) in [
        (Size::ExtraSmall, 24.0),
        (Size::Small, 30.0),
        (Size::Medium, 36.0),
        (Size::Large, 42.0),
        (Size::ExtraLarge, 50.0),
    ] {
        let app = App::memory();
        app.mount(button().size(size).label("Save")).unwrap();
        assert_eq!(
            app.tree()
                .node(root(&app))
                .unwrap()
                .descriptor
                .view()
                .style
                .min_height,
            Some(Length::Px(height))
        );
        app.clear().unwrap();
        app.mount(
            sapid::action_icon("Save")
                .size(size)
                .icon(text().content("+")),
        )
        .unwrap();
        let tree = app.tree();
        let style = tree.node(root(&app)).unwrap().descriptor.view().style;
        assert_eq!(style.width, Some(Length::Px(height)));
        assert_eq!(style.height, Some(Length::Px(height)));
    }
}
#[test]
fn sections_preserve_label_identity_and_exclude_adornment_text_from_the_button_name() {
    let app = App::memory();
    let source = app.scope().cage("Save".to_string());
    app.mount(
        button()
            .label(source)
            .left_section(text().content("★"))
            .right_section(text().content("Ctrl S")),
    )
    .unwrap();
    let node = root(&app);
    let text = label(&app);
    assert_eq!(
        app.tree()
            .node(node)
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref(),
        Some("Save")
    );
    source.revise(|s| *s = " ".into());
    assert!(app.flush().is_err());
    assert_eq!(
        app.tree()
            .node(node)
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref(),
        Some("Save")
    );
    source.revise(|s| *s = "Save changes".into());
    app.flush().unwrap();
    assert_eq!(label(&app), text);
    assert!(
        matches!(app.tree().node(text).unwrap().descriptor.view().props,Props::Text(p)if p.content=="Save changes")
    );
    assert_eq!(
        app.tree()
            .node(node)
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref(),
        Some("Save changes")
    );
}
#[test]
fn section_validation_owns_its_dependency_inside_keyed_construction_and_keeps_typed_errors() {
    let app = App::memory();
    let rows = app.scope().cage(vec!["Save".to_owned()]);
    app.mount(each(
        rows,
        |_| 0u8,
        |item, _| {
            widget(move |scope| {
                button()
                    .label(scope.bond(move || item.get()))
                    .left_section(text().content("★"))
                    .show_in(scope)
            })
        },
    ))
    .unwrap();
    let stable = label(&app);
    rows.revise(|r| r[0] = "Changed".into());
    app.flush().unwrap();
    assert_eq!(label(&app), stable);
    assert!(
        matches!(app.tree().node(stable).unwrap().descriptor.view().props,Props::Text(p)if p.content=="Changed")
    );
    let app = App::memory();
    let error = app
        .mount(each(
            vec!["Invalid".to_owned()],
            |_| 0u8,
            |item, _| {
                widget(move |scope| {
                    button()
                        .label(scope.bond(move || item.get()))
                        .left_section(view())
                        .show_in(scope)
                })
            },
        ))
        .unwrap_err();
    assert!(
        matches!(
            error,
            glory::ui::Error::Render(glory::render::Error::InvalidProps(
                "button children must be inline text, images or groups"
            ))
        ),
        "{error:?}"
    );
}
#[test]
fn section_names_and_shared_button_content_rules_reject_before_leaving_nodes() {
    let app = App::memory();
    assert!(
        app.mount(button().left_section(text().content("+")))
            .is_err()
    );
    assert!(app.tree().is_empty());
    assert!(
        app.mount(button().label("Bad").left_section(input()))
            .is_err()
    );
    assert!(app.tree().is_empty());
    app.mount(button().name("Upload").left_section(text().content("↑")))
        .unwrap();
    assert_eq!(
        app.tree()
            .node(root(&app))
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref(),
        Some("Upload")
    );
}
#[test]
fn loading_has_busy_metadata_and_disables_without_losing_label_or_identity() {
    let app = App::memory();
    let loading = app.scope().cage(false);
    app.mount(button().label("Save").loading(loading)).unwrap();
    let id = root(&app);
    for value in [true, false] {
        loading.revise(|old| *old = value);
        app.flush().unwrap();
        let tree = app.tree();
        let d = tree.node(id).unwrap().descriptor.view();
        assert_eq!(d.semantics.as_ref().unwrap().busy, Some(value));
        assert!(matches!(d.props,Props::Button(p)if p.label=="Save"&&p.enabled!=value));
    }
}
#[test]
fn disabled_state_uses_portable_theme_colors_and_preserves_explicit_declarations() {
    let app = App::memory();
    let loading = app.scope().cage(false);
    let theme = sapid::Theme::green_dark();
    app.mount(
        button()
            .theme(theme.clone())
            .label("Save")
            .loading(loading)
            .style(Style::new().border_color(Color(1, 2, 3, 255))),
    )
    .unwrap();
    let id = root(&app);
    loading.revise(|value| *value = true);
    app.flush().unwrap();
    {
        let tree = app.tree();
        let style = tree.node(id).unwrap().descriptor.view().style;
        assert_eq!(style.background, Some(theme.disabled));
        assert_eq!(style.color, Some(theme.disabled_text));
        assert_eq!(style.border_color, Some(Color(1, 2, 3, 255)));
    }
    loading.revise(|value| *value = false);
    app.flush().unwrap();
    assert_eq!(
        app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .style
            .background,
        Some(theme.accent)
    );
}
#[test]
fn retiring_in_a_typed_callback_skips_later_callbacks_and_generic_observers() {
    let app = App::memory();
    let mounted = Rc::new(RefCell::new(None::<glory::ui::Mounted>));
    let retire = mounted.clone();
    let order = Rc::new(RefCell::new(Vec::new()));
    let first = order.clone();
    let later = order.clone();
    let generic = order.clone();
    let handle = app
        .scope()
        .mount_with(|| {
            button()
                .label("Retire")
                .on(EventKind::Activate, move |_| {
                    generic.borrow_mut().push("generic")
                })
                .on_press(move || {
                    first.borrow_mut().push("first");
                    let handle = retire.borrow_mut().take();
                    drop(handle);
                })
                .on_press(move || later.borrow_mut().push("later"))
        })
        .unwrap();
    *mounted.borrow_mut() = Some(handle);
    app.flush().unwrap();
    app.dispatch(root(&app), Event::Activate).unwrap();
    assert_eq!(*order.borrow(), ["first"]);
    assert!(app.tree().is_empty());
}
#[test]
fn typed_callbacks_precede_generic_observers_regardless_of_setter_order() {
    let app = App::memory();
    let order = Rc::new(RefCell::new(Vec::new()));
    let a = order.clone();
    let b = order.clone();
    let c = order.clone();
    let d = order.clone();
    app.mount(
        button()
            .label("Save")
            .on(EventKind::Activate, move |_| {
                a.borrow_mut().push("generic1")
            })
            .on_press(move || b.borrow_mut().push("typed1"))
            .on(EventKind::Activate, move |_| {
                c.borrow_mut().push("generic2")
            })
            .on_press(move || d.borrow_mut().push("typed2")),
    )
    .unwrap();
    app.dispatch(root(&app), Event::Activate).unwrap();
    assert_eq!(
        *order.borrow(),
        ["typed1", "typed2", "generic1", "generic2"]
    );
}
#[test]
fn section_ssr_uses_inline_flex_even_unstyled_and_keeps_escaped_text() {
    let html = glory::ssr::render(
        button()
            .unstyled()
            .label("Save <draft>")
            .left_section(text().content("★"))
            .right_section(text().content("Ctrl+S")),
    )
    .unwrap();
    assert!(html.contains("inline-flex"));
    assert!(html.contains("Save &lt;draft&gt;"));
    assert!(html.contains("aria-busy=\"false\""));
    assert!(html.contains("sapid-button-label"));
}
