use glory::{style::Color, ui::App};
use sapid::{BLUE, ColorScheme, GREEN, Palette, PaletteOverrides, Theme, ThemeDefinition};

#[test]
fn shipped_css_resolves_both_definitions_from_the_same_base_and_overrides_as_widgets() {
    assert_eq!(Theme::default().scheme, BLUE.default().0);
    let css = include_str!("../styles/sapid.css");
    let mut rules = String::from("/* BEGIN SAPID PALETTES */\n");
    for definition in [&BLUE, &GREEN] {
        assert_eq!(definition.default(), (ColorScheme::Light, definition.base));
        assert!(
            definition.light.is_none(),
            "default Light needs no duplicate override"
        );
        for scheme in [ColorScheme::Light, ColorScheme::Dark] {
            let colors = definition.resolve(scheme).unwrap();
            let theme = Theme::new(definition, scheme).unwrap();
            assert_eq!(theme.scheme, scheme);
            assert_eq!(theme.palette(), &colors.shades);
            rules.push_str(&theme.stylesheet().unwrap());
        }
    }
    rules.push_str("/* END SAPID PALETTES */");
    assert!(css.contains(&rules), "run cargo run --example palette_css");
}

#[test]
fn every_base_color_is_customizable_and_construction_has_no_light_or_dark_replacements() {
    let base = Palette {
        shades: [Color(11, 12, 13, 14); 10],
        canvas: Color(1, 2, 3, 4),
        surface: Color(5, 6, 7, 8),
        input: Color(9, 10, 11, 12),
        foreground: Color(13, 14, 15, 16),
        muted: Color(17, 18, 19, 20),
        accent: Color(21, 22, 23, 24),
        accent_hover: Color(25, 26, 27, 28),
        accent_soft: Color(29, 30, 31, 32),
        accent_text: Color(33, 34, 35, 36),
        on_accent: Color(37, 38, 39, 40),
        focus_ring: Color(41, 42, 43, 44),
        input_focus: Color(45, 46, 47, 48),
        danger: Color(49, 50, 51, 52),
        danger_fill: Color(53, 54, 55, 56),
        on_danger: Color(57, 58, 59, 60),
        danger_soft: Color(61, 62, 63, 64),
        border: Color(65, 66, 67, 68),
        disabled: Color(69, 70, 71, 72),
        disabled_text: Color(73, 74, 75, 76),
        ..BLUE.base
    };
    let definition =
        ThemeDefinition::new("application-42", base).with_dark(PaletteOverrides::NONE);
    for scheme in [ColorScheme::Light, ColorScheme::Dark] {
        let theme = Theme::new(&definition, scheme).unwrap();
        assert_eq!(theme.shades, base.shades);
        let actual = [
            theme.canvas,
            theme.surface,
            theme.input,
            theme.foreground,
            theme.muted,
            theme.accent,
            theme.accent_hover,
            theme.accent_soft,
            theme.accent_text,
            theme.on_accent,
            theme.focus_ring,
            theme.input_focus,
            theme.danger,
            theme.danger_fill,
            theme.on_danger,
            theme.danger_soft,
            theme.border,
            theme.disabled,
            theme.disabled_text,
        ];
        let expected: [Color; 19] = std::array::from_fn(|i| {
            let n = (i * 4 + 1) as u8;
            Color(n, n + 1, n + 2, n + 3)
        });
        assert_eq!(actual, expected, "{scheme:?}");
        let css = theme.stylesheet().unwrap();
        assert!(css.contains("--sp-canvas:#01020304;"));
        assert!(css.contains("--sp-surface:#05060708;"));
        assert!(css.contains("--sp-text:#0d0e0f10;"));
        assert!(css.contains("--sp-border:#41424344;"));
        assert!(css.contains("--sp-danger:#31323334;"));
        assert!(css.contains("--sp-danger-fill:#35363738;"));
        assert!(css.contains("--sp-on-danger:#393a3b3c;"));
        assert!(css.contains("--sp-danger-soft:#3d3e3f40;"));
        assert!(css.contains("--sp-ring:#292a2b2c;"));
        assert!(css.contains("--sp-input-focus:#2d2e2f30;"));
        let app = App::memory();
        app.mount(
            sapid::theme_provider(theme.clone())
                .fill(sapid::button().theme(theme).label("Custom")),
        )
        .unwrap();
        let tree = app.tree();
        let root = tree.node(tree.roots()[0]).unwrap();
        assert_eq!(root.descriptor.view().style.background, Some(base.canvas));
        let button = tree.node(root.children[0]).unwrap().descriptor.view();
        assert_eq!(button.style.background, Some(base.accent));
        assert!(
            button
                .classes
                .iter()
                .any(|name| name == &format!("sapid-theme-application-42-{}", scheme.name()))
        );
    }
}

#[test]
fn light_and_dark_overrides_are_sparse_independent_and_preserve_transparent_values() {
    let base = BLUE.base;
    let light = Color(1, 2, 3, 4);
    let dark = Color(5, 6, 7, 8);
    let definition = ThemeDefinition::new("independent", base)
        .with_light(PaletteOverrides {
            canvas: Some(light),
            border: Some(Color(0, 0, 0, 0)),
            ..PaletteOverrides::NONE
        })
        .with_dark(PaletteOverrides {
            foreground: Some(dark),
            shades: [Some(dark); 10],
            ..PaletteOverrides::NONE
        });
    let l = definition.light().unwrap();
    let d = definition.dark().unwrap();
    assert_eq!(definition.default(), (ColorScheme::Light, l));
    assert_eq!(
        (l.canvas, l.border, l.foreground, l.shades),
        (light, Color(0, 0, 0, 0), base.foreground, base.shades)
    );
    assert_eq!(
        (d.canvas, d.border, d.foreground, d.shades),
        (base.canvas, base.border, dark, [dark; 10])
    );
    assert_eq!(definition.light(), Some(l));
    assert_eq!(definition.base, base);
    let theme = Theme::new(&definition, ColorScheme::Light).unwrap();
    assert!(
        theme
            .stylesheet()
            .unwrap()
            .contains("--sp-border:#00000000;")
    );
}

#[test]
fn default_mode_requires_no_override_and_other_modes_require_explicit_support() {
    let light = ThemeDefinition::new("light-only", GREEN.base);
    assert_eq!(light.default(), (ColorScheme::Light, GREEN.base));
    assert_eq!(light.light(), Some(GREEN.base));
    assert_eq!(light.dark(), None);
    assert!(Theme::new(&light, ColorScheme::Dark).is_err());
    assert_eq!(
        Theme::from_default(&light).unwrap().scheme,
        ColorScheme::Light
    );
    let dark =
        ThemeDefinition::new("dark-only", GREEN.base).with_default_scheme(ColorScheme::Dark);
    assert_eq!(dark.default(), (ColorScheme::Dark, GREEN.base));
    assert_eq!(dark.dark(), Some(GREEN.base));
    assert_eq!(dark.light(), None);
    assert!(Theme::new(&dark, ColorScheme::Light).is_err());
    assert_eq!(
        Theme::from_default(&dark).unwrap().surface,
        GREEN.base.surface
    );
    let both = dark.with_light(PaletteOverrides::default());
    assert_eq!(both.light(), Some(GREEN.base));
    assert_eq!(both.dark(), Some(GREEN.base));
}

#[test]
fn all_color_overrides_are_applied_before_typed_styles_and_css_are_built() {
    let color = Color(82, 35, 123, 192);
    let mut overrides = PaletteOverrides {
        shades: [Some(color); 10],
        series: [Some(color); 8],
        ..PaletteOverrides::NONE
    };
    for name in Palette::COLOR_NAMES {
        assert!(overrides.set_color(name, Some(color)));
    }
    let definition = ThemeDefinition::new("all-overrides", BLUE.base)
        .with_dark(overrides)
        .with_default_scheme(ColorScheme::Dark);
    let (scheme, palette) = definition.default();
    assert_eq!(scheme, ColorScheme::Dark);
    assert_eq!(palette, definition.dark().unwrap());
    let theme = Theme::new(&definition, scheme).unwrap();
    assert_eq!(
        [
            theme.canvas,
            theme.surface,
            theme.input,
            theme.foreground,
            theme.muted,
            theme.accent,
            theme.accent_hover,
            theme.accent_soft,
            theme.accent_text,
            theme.on_accent,
            theme.focus_ring,
            theme.input_focus,
            theme.danger,
            theme.danger_fill,
            theme.on_danger,
            theme.danger_soft,
            theme.border,
            theme.disabled,
            theme.disabled_text
        ],
        [color; 19]
    );
    assert_eq!(theme.shades, [color; 10]);
    let css = theme.stylesheet().unwrap();
    assert_eq!(
        css.matches(":#52237bc0;").count(),
        Palette::COLOR_NAMES.len() + 18
    );
    for name in Palette::COLOR_NAMES {
        assert_eq!(theme.colors().color(name), Some(color), "{name}");
    }
    assert_eq!(theme.series, [color; 8]);
}

#[test]
fn invalid_identifiers_reject_before_construction_and_provider_or_stylesheet_output() {
    for name in [
        "",
        "A",
        "0brand",
        "two words",
        "brand;}",
        "brand\n",
        "é",
        "-brand",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    ] {
        let definition = ThemeDefinition::new(name, GREEN.base);
        assert!(Theme::from_default(&definition).is_err(), "{name:?}");
        let mut theme = Theme::green_light();
        theme.name = name.into();
        assert!(theme.stylesheet().is_err(), "{name:?}");
        let app = App::memory();
        assert!(
            app.mount(sapid::theme_provider(theme).fill(sapid::button().label("Invalid")))
                .is_err()
        );
        assert_eq!(app.stats().nodes, 0);
    }
}

#[test]
fn indexed_overrides_inherit_future_base_changes_independently() {
    let mut base = BLUE.base;
    let mut overrides = PaletteOverrides::NONE;
    overrides.shades[2] = Some(Color(1, 2, 3, 0));
    overrides.series[4] = Some(Color(5, 6, 7, 8));
    base.shades[3] = Color(9, 10, 11, 12);
    base.series[3] = Color(13, 14, 15, 16);
    let result = overrides.apply(base);
    assert_eq!(result.shades[2], Color(1, 2, 3, 0));
    assert_eq!(result.shades[3], base.shades[3]);
    assert_eq!(result.series[4], Color(5, 6, 7, 8));
    assert_eq!(result.series[3], base.series[3]);
    assert!(!base.set_color("unknown", Color(0, 0, 0, 0)));
    assert!(!overrides.set_color("unknown", None));
}

#[test]
fn formerly_embedded_status_shadow_and_checker_colors_follow_custom_roles() {
    use glory::{
        prelude::*,
        render::Props,
        style::{BoxShadow, Shadow},
    };
    let mut colors = BLUE.base;
    let custom = Color(17, 39, 81, 123);
    colors.success = custom;
    colors.warning = custom;
    colors.success_soft = custom;
    colors.warning_soft = custom;
    colors.shadow_medium = custom;
    colors.checker_light = custom;
    colors.checker_dark = custom;
    let theme = Theme::from_palette("custom-colors", ColorScheme::Dark, colors).unwrap();
    let app = App::memory();
    app.mount(
        group().fill((
            sapid::alert("success")
                .theme(theme.clone())
                .severity(sapid::AlertSeverity::Success)
                .content("ok"),
            sapid::alert("warning")
                .theme(theme.clone())
                .severity(sapid::AlertSeverity::Warning)
                .content("careful"),
            sapid::paper()
                .theme(theme.clone())
                .shadow(sapid::ShadowSize::Medium),
            sapid::color_swatch(Color(0, 0, 0, 0)).theme(theme),
        )),
    )
    .unwrap();
    let mut pending = app.tree().roots().to_vec();
    let mut alerts = 0;
    let mut papers = 0;
    let mut checkers = 0;
    while let Some(id) = pending.pop() {
        let tree = app.tree();
        let n = tree.node(id).unwrap();
        let d = n.descriptor.view();
        pending.extend(n.children.iter().copied());
        if d.classes.iter().any(|c| c == "sapid-alert") {
            assert_eq!(d.style.background, Some(custom));
            assert_eq!(d.style.color, Some(custom));
            alerts += 1;
        }
        if d.classes.iter().any(|c| c == "sapid-paper") {
            assert!(
                matches!(d.style.box_shadow,Some(BoxShadow::Outset(Shadow{color,..})) if color==custom)
            );
            papers += 1;
        }
        if d.classes.iter().any(|c| c == "sapid-color-swatch-checker") {
            assert!(matches!(d.props,Props::Graphic(p) if p.fill==Some(custom)));
            checkers += 1;
        }
    }
    assert_eq!((alerts, papers, checkers), (2, 1, 1));
}
