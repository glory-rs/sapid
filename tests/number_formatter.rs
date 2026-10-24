use glory::render::{NodeId, Props};
use glory::style::Style;
use glory::ui::App;
use sapid::{Currency, NumberFormat, NumberLocale, Theme, number_formatter};

fn root(app: &App) -> NodeId {
    app.tree().roots()[0]
}
fn text(app: &App) -> String {
    let Props::Text(text) = app
        .tree()
        .node(root(app))
        .unwrap()
        .descriptor
        .view()
        .props
        .clone()
    else {
        panic!("expected text")
    };
    text.content
}
fn formatted(value: f64, locale: NumberLocale, format: NumberFormat) -> String {
    let app = App::memory();
    app.mount(
        number_formatter()
            .value(value)
            .locale(locale)
            .format(format),
    )
    .unwrap();
    text(&app)
}

#[test]
fn locales_and_decimal_grouping_are_explicit() {
    assert_eq!(
        formatted(1234567.125, NumberLocale::EnUs, NumberFormat::Decimal),
        "1,234,567.125"
    );
    assert_eq!(
        formatted(-1234567.125, NumberLocale::DeDe, NumberFormat::Decimal),
        "-1.234.567,125"
    );
    assert_eq!(
        formatted(1234567.125, NumberLocale::ZhCn, NumberFormat::Decimal),
        "1,234,567.125"
    );
    let app = App::memory();
    app.mount(number_formatter().value(1234.0).grouping(false))
        .unwrap();
    assert_eq!(text(&app), "1234");
}

#[test]
fn every_currency_has_explicit_symbol_and_position_in_each_locale() {
    for (locale, usd, eur, cny) in [
        (NumberLocale::EnUs, "$1,234.50", "€1,234.50", "CN¥1,234.50"),
        (
            NumberLocale::DeDe,
            "1.234,50\u{a0}US$",
            "1.234,50\u{a0}€",
            "1.234,50\u{a0}CN¥",
        ),
        (NumberLocale::ZhCn, "US$1,234.50", "€1,234.50", "¥1,234.50"),
    ] {
        for (currency, expected) in [
            (Currency::Usd, usd),
            (Currency::Eur, eur),
            (Currency::Cny, cny),
        ] {
            assert_eq!(
                formatted(1234.5, locale, NumberFormat::Currency(currency)),
                expected
            );
        }
    }
    assert_eq!(
        formatted(
            -1.0,
            NumberLocale::EnUs,
            NumberFormat::Currency(Currency::Usd)
        ),
        "-$1.00"
    );
}

#[test]
fn percent_scales_fraction_and_uses_locale_spacing() {
    assert_eq!(
        formatted(0.125, NumberLocale::EnUs, NumberFormat::Percent),
        "12%"
    );
    assert_eq!(
        formatted(-0.25, NumberLocale::DeDe, NumberFormat::Percent),
        "-25\u{a0}%"
    );
    let app = App::memory();
    app.mount(
        number_formatter()
            .value(0.125)
            .format(NumberFormat::Percent)
            .min_fraction_digits(1)
            .max_fraction_digits(2),
    )
    .unwrap();
    assert_eq!(text(&app), "12.5%");
}

#[test]
fn fraction_budgets_round_trim_and_retain_required_digits() {
    for (value, min, max, expected) in [
        (1234.125, 0, 2, "1,234.12"),
        (1234.375, 0, 2, "1,234.38"),
        (1.5, 3, 5, "1.500"),
        (9.999, 0, 2, "10"),
        (0.0, 2, 2, "0.00"),
        (-0.0, 1, 1, "-0.0"),
    ] {
        let app = App::memory();
        app.mount(
            number_formatter()
                .value(value)
                .min_fraction_digits(min)
                .max_fraction_digits(max),
        )
        .unwrap();
        assert_eq!(text(&app), expected);
    }
}

#[test]
fn invalid_static_values_and_fraction_budgets_fail_before_creation() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let app = App::memory();
        assert!(app.mount(number_formatter().value(value)).is_err());
        assert_eq!(app.tree().len(), 0);
    }
    for (min, max) in [(1, 0), (19, 19), (0, 255)] {
        let app = App::memory();
        assert!(
            app.mount(
                number_formatter()
                    .min_fraction_digits(min)
                    .max_fraction_digits(max)
            )
            .is_err()
        );
        assert_eq!(app.tree().len(), 0);
    }
    let app = App::memory();
    assert!(
        app.mount(
            number_formatter()
                .value(f64::MAX)
                .format(NumberFormat::Percent)
        )
        .is_err()
    );
    assert_eq!(app.tree().len(), 0);
}

#[test]
fn extremes_have_bounded_output_and_keep_all_finite_integer_digits() {
    let output = formatted(f64::MAX, NumberLocale::EnUs, NumberFormat::Decimal);
    assert!(output.starts_with("179,769,313,486"));
    assert_eq!(output.chars().filter(char::is_ascii_digit).count(), 309);
    assert!(output.len() < 450);
    let app = App::memory();
    app.mount(
        number_formatter()
            .value(f64::MIN_POSITIVE)
            .min_fraction_digits(18)
            .max_fraction_digits(18),
    )
    .unwrap();
    assert_eq!(text(&app), "0.000000000000000000");
}

#[test]
fn reactive_updates_are_silent_preserve_identity_and_never_write_source() {
    let app = App::memory();
    let value = app.scope().cage(10.125);
    app.mount(number_formatter().value(value)).unwrap();
    let id = root(&app);
    let before = app.tree().stats().patched;
    value.revise(|v| *v = 1234.5);
    app.flush().unwrap();
    assert_eq!(text(&app), "1,234.5");
    assert_eq!(value.get(), 1234.5);
    assert_eq!(root(&app), id);
    assert_eq!(app.stats().handlers, 0);
    assert!(app.tree().stats().patched > before);
    let before = app.tree().stats().patched;
    value.revise(|v| *v = 1234.5001);
    app.flush().unwrap();
    assert_eq!(value.get(), 1234.5001);
    assert_eq!(app.tree().stats().patched, before);
}

#[test]
fn nonfinite_reactive_update_preserves_last_accepted_text_then_recovers() {
    let app = App::memory();
    let value = app.scope().cage(12.5);
    app.mount(number_formatter().value(value)).unwrap();
    let id = root(&app);
    value.revise(|v| *v = f64::NAN);
    assert!(app.flush().is_err());
    assert_eq!(text(&app), "12.5");
    assert!(value.get().is_nan());
    value.revise(|v| *v = 25.5);
    app.flush().unwrap();
    assert_eq!(text(&app), "25.5");
    assert_eq!(root(&app), id);
}

#[test]
fn themes_and_unstyled_preserve_override_and_passive_semantics() {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        let app = App::memory();
        app.mount(
            number_formatter()
                .theme(theme.clone())
                .style(Style::new().radius(3.0)),
        )
        .unwrap();
        let d = app
            .tree()
            .node(root(&app))
            .unwrap()
            .descriptor
            .view()
            .to_owned();
        assert_eq!(d.style.color, Some(theme.foreground));
        assert_eq!(d.style.radius, Some(3.0));
        assert_eq!(d.name, None);
        assert_eq!(d.role, None);
    }
    let app = App::memory();
    app.mount(
        number_formatter()
            .unstyled()
            .style(Style::new().radius(3.0)),
    )
    .unwrap();
    assert_eq!(
        app.tree()
            .node(root(&app))
            .unwrap()
            .descriptor
            .view()
            .style
            .clone(),
        Style::new().radius(3.0)
    );
}

#[test]
fn formatter_bindings_and_refs_retire_with_owner() {
    let app = App::memory();
    let value = app.scope().cage(1.0);
    let reference = app.scope().cage(None);
    let before = app.runtime_stats();
    let mounted = app
        .scope()
        .mount_with(|| number_formatter().value(value).node_ref(reference))
        .unwrap();
    app.flush().unwrap();
    assert!(reference.get().is_some());
    drop(mounted);
    app.flush().unwrap();
    assert_eq!(reference.get(), None);
    assert_eq!(app.runtime_stats().edges, before.edges);
    value.revise(|v| *v = 2.0);
    app.flush().unwrap();
    assert_eq!(app.tree().len(), 0);
}

#[test]
fn ssr_uses_the_same_locale_output_and_escapes_user_classes() {
    let html = glory::ssr::render(
        number_formatter()
            .value(1234.5)
            .format(NumberFormat::Currency(Currency::Eur))
            .locale(NumberLocale::DeDe)
            .class("a<&"),
    )
    .unwrap();
    assert!(html.contains("1.234,50\u{a0}€"));
    assert!(html.contains("a&lt;&amp;"));
    assert!(!html.contains("<script"));
}
