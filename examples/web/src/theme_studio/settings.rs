//! Generation, mode support and the three explicit edit layers.
use super::*;
#[allow(clippy::too_many_arguments)]
pub(super) fn controls(
    draft: Cage<Draft>,
    layer: Cage<Layer>,
    mode: Cage<ColorScheme>,
    target: Cage<Target>,
    picker: Cage<Hsva>,
    status: Cage<String>,
    theme: Theme,
) -> impl Widget {
    widget(move |scope| {
        let seed = scope.bond(move || draft.with(|d| d.seed.clone()));
        let generate = view().class("studio-card").fill((
            section("Generate a tonal ramp"),
            copy("studio-help", "Ten perceptual Oklab tones. The seed stays at index 6; gamut mapping reduces chroma. Generation replaces accent roles and preserves neutral colors."),
            view().class("studio-seed-row").fill((
                sapid::text_field().theme(theme.clone()).name("Seed color").value(seed)
                    .on_value_change(move |v| draft.revise(|d| d.seed = v.chars().take(32).collect())),
                sapid::button().theme(theme.clone()).label("Generate accent colors").on_press(move || {
                    let mut error = None;
                    draft.revise(|d| { if let Err(e) = d.generate() { error = Some(e); } });
                    status.revise(|s| *s = error.unwrap_or_else(|| "Accent ramp generated; neutral colors preserved.".into()));
                    update_picker(draft, layer, target, picker);
                }),
            )),
            view().class("studio-ramp").fill((0..10).map(|index| {
                let style = scope.bond(move || Style { background:Some(draft.with(|d|d.record.base.shades[index])), ..Style::new() });
                sapid::button().theme(theme.clone()).unstyled().class("studio-ramp-tone").style(style).label(index.to_string()).name(format!("Inspect generated shade {index}"))
                    .on_press(move || { layer.revise(|v|*v=Layer::Base);target.revise(|v|*v=Target::Shade(index));update_picker(draft,layer,target,picker); })
            }).collect::<Vec<_>>()),
        ));
        let support = view().class("studio-mode-controls").fill(
            [ColorScheme::Light, ColorScheme::Dark]
                .into_iter()
                .map(|scheme| {
                    let checked = scope.bond(move || draft.with(|d| d.record.supports(scheme)));
                    let enabled =
                        scope.bond(move || draft.with(|d| d.record.default_scheme != scheme));
                    sapid::checkbox()
                        .theme(theme.clone())
                        .name(format!("Support {} mode", scheme.name()))
                        .label(if scheme == ColorScheme::Light {
                            "Light mode"
                        } else {
                            "Dark mode"
                        })
                        .checked(checked)
                        .enabled(enabled)
                        .on_checked_change(move |on| {
                            draft.revise(|d| d.set_support(scheme, on));
                            if !draft.with(|d| d.record.supports(mode.get())) {
                                mode.revise(|m| *m = draft.with(|d| d.record.default_scheme));
                            }
                            update_picker(draft, layer, target, picker);
                        })
                })
                .collect::<Vec<_>>(),
        );
        let defaults = view().class("studio-mode-controls").fill(
            [ColorScheme::Light, ColorScheme::Dark]
                .into_iter()
                .map(|scheme| {
                    let class = scope.bond(move || {
                        if draft.with(|d| d.record.default_scheme == scheme) {
                            "studio-tab is-active".to_owned()
                        } else {
                            "studio-tab".to_owned()
                        }
                    });
                    sapid::button()
                        .theme(theme.clone())
                        .unstyled()
                        .class(class)
                        .label(if scheme == ColorScheme::Light {
                            "Default: Light"
                        } else {
                            "Default: Dark"
                        })
                        .on_press(move || {
                            draft.revise(|d| {
                                d.record.set_default(scheme);
                                d.epoch = d.epoch.saturating_add(1);
                            })
                        })
                })
                .collect::<Vec<_>>(),
        );
        let modes = view().class("studio-card").fill((
            section("Modes and inheritance"),
            copy("studio-help", "The default mode always exists. Empty overrides keep the base exactly; disabled alternate modes are omitted from export."), support, defaults,
        ));
        let tabs = view().class("studio-mode-controls").fill(
            [Layer::Base, Layer::Light, Layer::Dark]
                .into_iter()
                .map(|value| {
                    let class = scope.bond(move || {
                        if layer.get() == value {
                            "studio-tab is-active".to_owned()
                        } else {
                            "studio-tab".to_owned()
                        }
                    });
                    sapid::button()
                        .theme(theme.clone())
                        .unstyled()
                        .class(class)
                        .name(format!("Edit {} colors", value.name()))
                        .label(value.name())
                        .on_press(move || {
                            layer.revise(|l| *l = value);
                            update_picker(draft, layer, target, picker);
                        })
                })
                .collect::<Vec<_>>(),
        );
        let edit_layers = scope.bond(move || vec![layer.get()]);
        let row_theme = theme.clone();
        let fields = view().class("studio-card").fill((
            section("Edit every color"),
            copy("studio-help", "Hex inputs accept RGB or RGBA. Enable Override to replace a mode value; uncheck it to inherit. Shades and series inherit independently per index."),
            tabs,
            each(edit_layers).key(|v|*v).children(move |entry| { let value = entry.value;fields::role_rows(draft,value.get(),target,picker,row_theme.clone())}),
        ));
        group().fill((generate, modes, fields)).show_in(scope)
    })
}
