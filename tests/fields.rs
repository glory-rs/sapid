use glory::prelude::*;
use glory::render::NodeId;
use sapid::{Theme, checkbox, field, fieldset, text_field};

fn find(app: &App, class: &str) -> NodeId {
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        if node.descriptor.view().classes.iter().any(|c| c == class) {
            return id;
        }
        pending.extend(node.children.iter().copied());
    }
    panic!("missing {class}")
}
fn metadata(app: &App, id: NodeId) -> Semantics {
    *app.tree()
        .node(id)
        .unwrap()
        .descriptor
        .view()
        .semantics
        .clone()
        .unwrap()
}
#[test]
fn field_relations_states_and_dynamic_errors_preserve_control_and_value_revision() {
    let app = App::memory();
    let error = app.scope().cage(String::new());
    let readonly = app.scope().cage(false);
    app.mount(
        field("email")
            .label("Email")
            .description("Private")
            .error(error)
            .readonly(readonly)
            .required(true)
            .control(|context| text_field().field(context).default_value("draft")),
    )
    .unwrap();
    let input = find(&app, "sapid-text-field");
    let before = app.tree().node(input).unwrap().descriptor.view().to_owned();
    let meta = metadata(&app, input);
    assert!(meta.required && !meta.invalid && !meta.readonly);
    let label = find(&app, "sapid-field-label");
    assert_eq!(metadata(&app, label).label_for, meta.id);
    assert_eq!(meta.labelled_by, metadata(&app, label).id);
    assert!(metadata(&app, find(&app, "sapid-field-error")).hidden);
    error.revise(|v| *v = "Invalid email".into());
    readonly.revise(|v| *v = true);
    app.flush().unwrap();
    let meta = metadata(&app, input);
    assert!(meta.invalid && meta.readonly && meta.error_message.is_some());
    assert_eq!(
        app.tree()
            .node(input)
            .unwrap()
            .descriptor
            .view()
            .props
            .clone(),
        before.props
    );
    assert_eq!(find(&app, "sapid-text-field"), input);
    let revision = if let glory::render::Props::Input(p) = before.props {
        p.revision
    } else {
        unreachable!()
    };
    assert!(
        !app.dispatch(
            input,
            Event::ValueChanged {
                value: "mutated".into(),
                composing: false,
                revision
            }
        )
        .unwrap()
    );
    error.revise(|v| v.clear());
    app.flush().unwrap();
    assert!(!metadata(&app, input).invalid);
}
#[test]
fn readonly_check_and_disabled_fieldset_block_user_changes_but_program_writes_work() {
    let app = App::memory();
    let readonly = app.scope().cage(true);
    let disabled = app.scope().cage(false);
    let value = app.scope().cage(false);
    app.mount(
        fieldset("group")
            .legend("Preferences")
            .disabled(disabled)
            .fill(
                field("consent")
                    .label("Consent")
                    .readonly(readonly)
                    .control(move |context| {
                        checkbox()
                            .field(context)
                            .checked(value)
                            .on_checked_change(move |v| value.revise(|old| *old = v))
                    }),
            ),
    )
    .unwrap();
    let check = find(&app, "sapid-checkbox");
    assert!(!app.dispatch(check, Event::CheckedChanged(true)).unwrap());
    assert!(!value.get());
    value.revise(|v| *v = true);
    app.flush().unwrap();
    assert!(
        matches!(&app.tree().node(check).unwrap().descriptor.view().props.clone(),glory::render::Props::Check(p) if p.checked)
    );
    readonly.revise(|v| *v = false);
    disabled.revise(|v| *v = true);
    app.flush().unwrap();
    assert!(!app.tree().enabled(check));
    assert!(!app.dispatch(check, Event::CheckedChanged(false)).unwrap());
    assert!(value.get());
    disabled.revise(|v| *v = false);
    app.flush().unwrap();
    assert!(app.dispatch(check, Event::CheckedChanged(false)).unwrap());
    assert!(!value.get());
}
#[test]
fn explicit_name_overrides_visible_label_and_teardown_releases_all_field_parts() {
    let app = App::memory();
    app.mount(
        field("named")
            .label("Visible label")
            .control(|context| text_field().field(context).name("Explicit name")),
    )
    .unwrap();
    let id = find(&app, "sapid-text-field");
    assert!(metadata(&app, id).labelled_by.is_none());
    assert_eq!(
        app.tree()
            .node(id)
            .unwrap()
            .descriptor
            .view()
            .name
            .as_deref(),
        Some("Explicit name")
    );
    app.clear().unwrap();
    assert_eq!(app.tree().len(), 0);
    assert!(app.mount(field("missing")).is_err());
    assert_eq!(app.tree().len(), 0);
}
#[test]
fn required_readonly_invalid_are_independent_from_validation_and_enabled_state() {
    let app = App::memory();
    app.mount(text_field().required(true).readonly(true).invalid(true))
        .unwrap();
    let id = find(&app, "sapid-text-field");
    let meta = metadata(&app, id);
    assert!(meta.required && meta.readonly && meta.invalid);
    assert!(app.tree().enabled(id));
    app.scope().set_input_value(id, "program").unwrap();
    app.flush().unwrap();
    assert!(
        matches!(&app.tree().node(id).unwrap().descriptor.view().props.clone(),glory::render::Props::Input(p) if p.value=="program")
    );
}
#[test]
fn ssr_uses_native_label_fieldset_and_error_relations_with_safe_text() {
    let snapshot = glory::ssr::snapshot(
        fieldset("profile")
            .legend("Profile <private>")
            .disabled(true)
            .fill(
                field("email")
                    .label("Email")
                    .description("Help")
                    .error("Wrong <value>")
                    .required(true)
                    .readonly(true)
                    .control(|context| text_field().field(context)),
            ),
    )
    .unwrap();
    for value in [
        "<fieldset",
        "<legend",
        "<label",
        "for=",
        "required",
        "readonly",
        "aria-invalid=\"true\"",
        "aria-errormessage",
        "disabled",
        "&lt;private&gt;",
        "&lt;value&gt;",
    ] {
        assert!(snapshot.html.contains(value), "{value}: {}", snapshot.html);
    }
}
fn luminance(color: Color) -> f64 {
    fn channel(x: u8) -> f64 {
        let x = f64::from(x) / 255.;
        if x <= 0.04045 {
            x / 12.92
        } else {
            ((x + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * channel(color.0) + 0.7152 * channel(color.1) + 0.0722 * channel(color.2)
}
fn contrast(a: Color, b: Color) -> f64 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}
#[test]
fn four_themes_have_distinct_surfaces_and_readable_semantic_text() {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        for (text, bg) in [
            (theme.foreground, theme.input),
            (theme.muted, theme.surface),
            (theme.danger, theme.surface),
            (theme.on_accent, theme.accent),
            (theme.on_danger, theme.danger_fill),
        ] {
            assert!(
                contrast(text, bg) >= 4.5,
                "{}: {:?}/{:?}",
                theme.class(),
                text,
                bg
            );
        }
        assert_eq!(theme.palette().len(), 10);
    }
    assert_ne!(Theme::blue_light().accent, Theme::green_light().accent);
    assert_ne!(Theme::blue_light().surface, Theme::blue_dark().surface);
}
