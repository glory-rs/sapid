//! Actual Sapid widget preview and resolved contrast observations.
use super::*;
pub(super) fn preview_panel(state: EditorState, theme: Theme) -> impl Widget {
    widget(move |scope| {
        let EditorState {
            draft,
            mode,
            clicks,
            sample_name: name,
            checked,
            ..
        } = state;
        let ticks = scope.bond(move || vec![(draft.with(|d| d.epoch), mode.get())]);
        let modes = view().class("studio-mode-controls").fill(
            [ColorScheme::Light, ColorScheme::Dark]
                .into_iter()
                .map(|scheme| {
                    let supported =
                        scope.bond(move || draft.with(|d| d.record.supports(scheme)));
                    let active = scope.bond(move || {
                        if mode.get() == scheme {
                            "studio-tab is-active".to_owned()
                        } else {
                            "studio-tab".to_owned()
                        }
                    });
                    sapid::button()
                        .theme(theme.clone())
                        .unstyled()
                        .class(active)
                        .name(format!("Preview {}", scheme.name()))
                        .label(if scheme == ColorScheme::Light {
                            "Light"
                        } else {
                            "Dark"
                        })
                        .enabled(supported)
                        .on_press(move || mode.revise(|m| *m = scheme))
                })
                .collect::<Vec<_>>(),
        );
        let samples = each(ticks)
            .key(|value| *value)
            .children(move |__each_row| sample(draft, mode, clicks, name, checked));
        let contrast = view().class("studio-contrast").fill(
            [
                ("Text / canvas", 0),
                ("Muted / surface", 1),
                ("Accent label / fill", 2),
                ("Danger label / fill", 3),
            ]
            .into_iter()
            .map(|(label, index)| {
                let value = scope.bond(move || {
                    draft.with(|d| {
                        let p = d.record.resolve(mode.get()).expect("supported preview");
                        let (a, b) = match index {
                            0 => (p.foreground, p.canvas),
                            1 => (p.muted, p.surface),
                            2 => (p.on_accent, p.accent),
                            _ => (p.on_danger, p.danger_fill),
                        };
                        format!("{label}: {:.2}:1", color::contrast(a, b))
                    })
                });
                copy("studio-contrast-item", value)
            })
            .collect::<Vec<_>>(),
        );
        view()
            .class("studio-preview-panel")
            .fill((
                section("Live preview"),
                copy(
                    "studio-help",
                    "Preview the actual Sapid widgets. Edits keep the sample's values.",
                ),
                modes,
                samples,
                section("Contrast checks"),
                copy(
                    "studio-help",
                    "Text contrast target: 4.5:1. Ratios assume an opaque background.",
                ),
                contrast,
            ))
            .show_in(scope)
    })
}
fn sample(
    draft: Cage<Draft>,
    mode: Cage<ColorScheme>,
    clicks: Cage<usize>,
    name: Cage<String>,
    checked: Cage<bool>,
) -> impl Widget {
    let mut theme = draft
        .with(|d| d.record.theme(mode.get()))
        .expect("supported preview");
    theme.name = "studio-preview".into();
    let content = (
        sapid::title(3)
            .theme(theme.clone())
            .content("Make it yours"),
        sapid::text()
            .theme(theme.clone())
            .content("Real widgets, using this exact definition."),
        sapid::button()
            .theme(theme.clone())
            .label("Preview action")
            .on_press(move || clicks.revise(|v| *v += 1)),
        sapid::text()
            .theme(theme.clone())
            .content(clicks.map(|v| format!("Clicked {v} times"))),
        sapid::text_field()
            .theme(theme.clone())
            .name("Preview name")
            .value(name)
            .on_value_change(move |v| name.revise(|s| *s = v)),
        sapid::checkbox()
            .theme(theme.clone())
            .name("Preview setting")
            .label("Enable notifications")
            .checked(checked)
            .on_checked_change(move |v| checked.revise(|s| *s = v)),
        sapid::alert("studio-success")
            .theme(theme.clone())
            .severity(sapid::AlertSeverity::Success)
            .title("Success")
            .content("This status color is a palette role."),
        sapid::alert("studio-warning")
            .theme(theme.clone())
            .severity(sapid::AlertSeverity::Warning)
            .title("Warning")
            .content("Its text and surface can both be tuned."),
    );
    view()
        .class(format!("studio-demo {}", theme.class()))
        .style(Style {
            background: Some(theme.canvas),
            color: Some(theme.foreground),
            ..Style::new().padding(Edges::all(22.)).radius(12.)
        })
        .fill(content)
}
