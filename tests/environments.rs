use glory::render::{Direction, Props};
use glory::ui::App;
use sapid::{Locale, LocaleMessage, Size, Theme, theme_provider};
use std::{cell::Cell, rc::Rc};
#[path = "../examples/web/environment_form.rs"]
mod fixture;

#[test]
fn explicit_factory_receives_locale_direction_and_theme_tokens_once() {
    let calls = Rc::new(Cell::new(0));
    let count = calls.clone();
    let theme = Theme::green_dark().with_tokens(
        sapid::ThemeTokens::default()
            .with_spacing(Size::Medium, 27.)
            .unwrap(),
    );
    let app = App::memory();
    app.mount(
        theme_provider(theme.clone())
            .direction(Direction::Rtl)
            .locale(Locale::arabic())
            .content(move |context| {
                count.set(count.get() + 1);
                assert_eq!(context.direction, Direction::Rtl);
                assert_eq!(context.spacing(Size::Medium), 27.);
                assert_eq!(context.locale.language().as_str(), "ar");
                sapid::button()
                    .theme(context.theme)
                    .label(context.locale.message(LocaleMessage::Close))
            }),
    )
    .unwrap();
    assert_eq!(calls.get(), 1);
    let tree = app.tree();
    let mut pending = tree.roots().to_vec();
    let mut found = [false; 2];
    while let Some(id) = pending.pop() {
        let node = tree.node(id).unwrap();
        let descriptor = node.descriptor.view();
        if let Some(env) = descriptor
            .semantics
            .as_ref()
            .and_then(|s| s.environment.as_ref())
        {
            assert_eq!(env.direction, Direction::Rtl);
            assert_eq!(env.language.as_str(), "ar");
            assert_eq!(descriptor.style.background, Some(theme.canvas));
            found[0] = true;
        }
        if let Props::Button(button) = descriptor.props {
            assert_eq!(button.label, "إغلاق");
            assert_eq!(
                descriptor.style.font_size,
                Some(
                    theme
                        .typography(sapid::TypographyRole::Control, Size::Medium)
                        .font_size
                )
            );
            found[1] = true;
        }
        pending.extend(node.children.iter().copied());
    }
    assert_eq!(found, [true; 2]);
    drop(tree);
    app.flush().unwrap();
    assert_eq!(calls.get(), 1);
}

#[test]
fn invalid_provider_configuration_rejects_before_the_content_factory() {
    let calls = Rc::new(Cell::new(0));
    let count = calls.clone();
    let mut theme = Theme::blue_light();
    theme.radius = f64::NAN;
    let app = App::memory();
    assert!(
        app.mount(theme_provider(theme).content(move |_| {
            count.set(1);
        }))
        .is_err()
    );
    assert_eq!(calls.get(), 0);
    assert!(app.tree().roots().is_empty());
}

#[test]
fn explicit_nested_context_preserves_values_and_reduce_motion_is_bounded() {
    let mut context = sapid::UiEnvironmentContext {
        direction: Direction::Rtl,
        locale: Locale::chinese(),
        ..Default::default()
    };
    context.motion = glory::render::MotionPreference::Reduce;
    assert_eq!(context.transition_duration(250).unwrap(), 0);
    assert!(context.transition_duration(2001).is_err());
    let app = App::memory();
    app.mount(context.provider().content(|context| {
        assert_eq!(context.message(LocaleMessage::Next), "下一个");
        context.provider().content(|nested| {
            assert_eq!(nested.direction, Direction::Rtl);
            sapid::text().content(nested.message(LocaleMessage::Loading))
        })
    }))
    .unwrap();
}

#[test]
fn locale_configuration_checks_each_bounded_label_before_installation() {
    let make = |close: String| sapid::LocaleMessages {
        open: "Open".into(),
        close,
        previous: "Previous".into(),
        next: "Next".into(),
        clear: "Clear".into(),
        search: "Search".into(),
        no_results: "None".into(),
        remove: "Remove".into(),
        cancel: "Cancel".into(),
        confirm: "Confirm".into(),
        loading: "Loading".into(),
        selected: "Selected".into(),
    };
    let language = glory::render::LanguageTag::new("en").unwrap();
    for invalid in ["x".repeat(1_000_000), " \t".into(), "Bad\nlabel".into()] {
        assert!(Locale::new(language.clone(), make(invalid)).is_err());
    }
    assert!(Locale::new(language, make("Close".into())).is_ok());
}

#[test]
fn provider_fixture_constructs_in_every_theme() {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        let app = App::memory();
        app.mount(fixture::environment_form(theme)).unwrap();
    }
}
