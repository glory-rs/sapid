use glory::prelude::*;
use glory::render::{Event, Props};
#[test]
fn text_disabled_colors_are_portable_and_state_changes_do_not_rewrite_values() {
    for theme in [
        sapid::Theme::blue_light(),
        sapid::Theme::blue_dark(),
        sapid::Theme::green_light(),
        sapid::Theme::green_dark(),
    ] {
        let app = App::memory();
        let enabled = app.scope().cage(true);
        app.mount(
            sapid::text_field()
                .theme(theme.clone())
                .enabled(enabled)
                .default_value("Retain this draft"),
        )
        .unwrap();
        let id = app.tree().roots()[0];
        let revision = match app.tree().node(id).unwrap().descriptor.view().props {
            Props::Input(p) => p.revision,
            _ => panic!(),
        };
        enabled.revise(|value| *value = false);
        app.flush().unwrap();
        {
            let tree = app.tree();
            let d = tree.node(id).unwrap().descriptor.view();
            assert_eq!(d.style.background, Some(theme.disabled));
            assert_eq!(d.style.color, Some(theme.disabled_text));
            assert!(
                matches!(d.props,Props::Input(p) if !p.enabled&&p.value=="Retain this draft"&&p.revision==revision)
            );
        }
        enabled.revise(|value| *value = true);
        app.flush().unwrap();
        assert_eq!(
            app.tree()
                .node(id)
                .unwrap()
                .descriptor
                .view()
                .style
                .background,
            Some(theme.input)
        );
        assert_eq!(
            app.tree().node(id).unwrap().descriptor.view().style.color,
            Some(theme.foreground)
        );
    }
}
#[test]
fn disabled_required_readonly_invalid_and_explicit_overrides_remain_independent() {
    let app = App::memory();
    let enabled = app.scope().cage(true);
    let readonly = app.scope().cage(false);
    let invalid = app.scope().cage(false);
    let theme = sapid::Theme::blue_dark();
    app.mount(
        sapid::text_field()
            .theme(theme.clone())
            .enabled(enabled)
            .readonly(readonly)
            .invalid(invalid)
            .required(true)
            .style(Style {
                color: Some(Color(1, 2, 3, 255)),
                ..Style::new()
            }),
    )
    .unwrap();
    let id = app.tree().roots()[0];
    enabled.revise(|value| *value = false);
    readonly.revise(|value| *value = true);
    invalid.revise(|value| *value = true);
    app.flush().unwrap();
    {
        let tree = app.tree();
        let d = tree.node(id).unwrap().descriptor.view();
        assert_eq!(d.style.color, Some(Color(1, 2, 3, 255)));
        assert_eq!(d.style.background, Some(theme.disabled));
        assert_eq!(d.style.border_color, Some(theme.danger));
        let s = d.semantics.as_ref().unwrap();
        assert!(s.required && s.readonly && s.invalid);
    }
    enabled.revise(|value| *value = true);
    readonly.revise(|value| *value = false);
    invalid.revise(|value| *value = false);
    app.flush().unwrap();
    assert_eq!(
        app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .style
            .border_color,
        Some(theme.border)
    );
}
#[test]
fn checkbox_disabled_label_colors_and_native_activation_recover_with_the_same_control() {
    let app = App::memory();
    let enabled = app.scope().cage(false);
    let theme = sapid::Theme::green_dark();
    app.mount(
        sapid::checkbox()
            .theme(theme.clone())
            .enabled(enabled)
            .label("Receive updates")
            .default_checked(true),
    )
    .unwrap();
    let id = app.tree().roots()[0];
    assert_eq!(
        app.tree().node(id).unwrap().descriptor.view().style.color,
        Some(theme.disabled_text)
    );
    assert!(!app.dispatch(id, Event::CheckedChanged(false)).unwrap());
    assert!(
        matches!(app.tree().node(id).unwrap().descriptor.view().props,Props::Check(p)if p.checked)
    );
    enabled.revise(|value| *value = true);
    app.flush().unwrap();
    assert_eq!(
        app.tree().node(id).unwrap().descriptor.view().style.color,
        Some(theme.foreground)
    );
    assert!(app.dispatch(id, Event::CheckedChanged(false)).unwrap());
    assert!(
        matches!(app.tree().node(id).unwrap().descriptor.view().props,Props::Check(p)if !p.checked)
    );
}
